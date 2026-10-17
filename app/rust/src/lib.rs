//! # jixel_bridge
//!
//! The only door between the Flutter app and the Rust engine. It is a tiny
//! C ABI with **one call**:
//!
//! ```c
//! JixelResponse* jixel_call(const char* method, const char* args_json,
//!                           const JixelBuffer* inputs, size_t n_inputs);
//! void jixel_free(JixelResponse*);
//! ```
//!
//! * `method` names an operation (`"convert"`, `"inspect"`, …; see
//!   [`dispatch`]).
//! * `args_json` carries its arguments. Most operations work on **file
//!   paths**: Rust reads and writes the files itself, so big images never
//!   cross the boundary twice.
//! * The response holds JSON (`{"ok": …}` or `{"error": "…"}`) plus optional
//!   binary buffers (e.g. RGBA pixels for on-screen previews), referred to by
//!   index from the JSON.
//!
//! Why not a code generator such as flutter_rust_bridge? One generic entry
//! point is ~200 readable lines, needs no generated code or extra tooling,
//! and keeps the Dart side in plain, typed classes. See
//! `docs/adr/0003-json-ffi-bridge.md`.
//!
//! Every call is synchronous; the Dart side runs it on a background isolate.

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};

use jixel_core::analyze::{compare, inspect, progressive};
use jixel_core::codec::jxl;
use jixel_core::convert::{self, decode_any, ConvertOptions};
use jixel_core::ops::background::{self, BackgroundOptions};
use jixel_core::ops::crop::{crop_file, CropMode};
use jixel_core::ops::resize::{self, Fit};
use jixel_core::ops::subject::{self, BoostOptions, Subject};
use jixel_core::ops::tiles::{self, Split};
use jixel_core::web::{self, WebExportOptions};
use jixel_core::{Format, Raster};

/// A byte buffer owned by whoever created it.
#[repr(C)]
pub struct JixelBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

/// The result of [`jixel_call`]. Free it with [`jixel_free`].
#[repr(C)]
pub struct JixelResponse {
    /// UTF-8 JSON, NUL-terminated.
    pub json: *mut c_char,
    pub buffers: *mut JixelBuffer,
    pub n_buffers: usize,
}

/// Runs one operation. Never panics across the boundary and never returns
/// null.
///
/// # Safety
/// `method` and `args_json` must be valid NUL-terminated UTF-8 strings;
/// `inputs` must point to `n_inputs` valid buffers (or be null when 0).
#[no_mangle]
pub unsafe extern "C" fn jixel_call(
    method: *const c_char,
    args_json: *const c_char,
    inputs: *const JixelBuffer,
    n_inputs: usize,
) -> *mut JixelResponse {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let method = CStr::from_ptr(method).to_str().map_err(|e| e.to_string())?;
        let args = CStr::from_ptr(args_json)
            .to_str()
            .map_err(|e| e.to_string())?;
        let args: Value = serde_json::from_str(args).map_err(|e| format!("bad arguments: {e}"))?;
        let inputs: Vec<&[u8]> = if n_inputs == 0 || inputs.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(inputs, n_inputs)
                .iter()
                .map(|b| std::slice::from_raw_parts(b.ptr as *const u8, b.len))
                .collect()
        };
        let mut out = Output::default();
        let value = dispatch(method, args, &inputs, &mut out).map_err(|e| format!("{e:#}"))?;
        Ok::<_, String>((value, out.buffers))
    }));
    let (json, buffers) = match outcome {
        Ok(Ok((value, buffers))) => (json!({ "ok": value }), buffers),
        Ok(Err(message)) => (json!({ "error": message }), Vec::new()),
        Err(panic) => {
            let message = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unexpected internal error".into());
            (
                json!({ "error": format!("internal error: {message}") }),
                Vec::new(),
            )
        }
    };
    let json = CString::new(json.to_string())
        .unwrap_or_default()
        .into_raw();
    let mut buffers: Vec<JixelBuffer> = buffers
        .into_iter()
        .map(|b| {
            let mut b = b.into_boxed_slice();
            let buf = JixelBuffer {
                ptr: b.as_mut_ptr(),
                len: b.len(),
            };
            std::mem::forget(b);
            buf
        })
        .collect();
    buffers.shrink_to_fit();
    let response = JixelResponse {
        json,
        n_buffers: buffers.len(),
        buffers: buffers.as_mut_ptr(),
    };
    std::mem::forget(buffers);
    Box::into_raw(Box::new(response))
}

/// Frees a response returned by [`jixel_call`].
///
/// # Safety
/// `response` must come from `jixel_call` and be freed only once.
#[no_mangle]
pub unsafe extern "C" fn jixel_free(response: *mut JixelResponse) {
    if response.is_null() {
        return;
    }
    let r = Box::from_raw(response);
    drop(CString::from_raw(r.json));
    let buffers = Vec::from_raw_parts(r.buffers, r.n_buffers, r.n_buffers);
    for b in buffers {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            b.ptr, b.len,
        )));
    }
}

/// Binary results of a call; the JSON refers to them by index.
#[derive(Default)]
struct Output {
    buffers: Vec<Vec<u8>>,
}

impl Output {
    fn push(&mut self, bytes: Vec<u8>) -> usize {
        self.buffers.push(bytes);
        self.buffers.len() - 1
    }

    /// Adds an on-screen preview: RGBA8 pixels, at most `max_side` px.
    fn preview(&mut self, raster: &Raster, max_side: u32) -> Result<Value> {
        let mut r = raster.clone();
        r.apply_orientation()?;
        if r.width.max(r.height) > max_side {
            r = resize::fit(&r, Fit::within(max_side, max_side))?;
        }
        let rgba = r.to_rgba8()?;
        let (width, height) = rgba.dimensions();
        let buffer = self.push(rgba.into_raw());
        Ok(json!({ "buffer": buffer, "width": width, "height": height }))
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn arg<T: for<'de> Deserialize<'de>>(args: &Value, key: &str) -> Result<T> {
    let v = args.get(key).cloned().unwrap_or(Value::Null);
    serde_json::from_value(v).map_err(|e| format!("argument '{key}': {e}").into())
}

fn opt<T: for<'de> Deserialize<'de>>(args: &Value, key: &str) -> Result<Option<T>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => arg(args, key).map(Some),
    }
}

fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()).into())
}

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()).into())
}

fn decode_path(path: &Path) -> Result<Raster> {
    Ok(decode_any(&read(path)?)?.raster)
}

/// Every operation the app can call. Arguments are documented per arm.
fn dispatch(method: &str, args: Value, inputs: &[&[u8]], out: &mut Output) -> Result<Value> {
    let path = |key: &str| -> Result<PathBuf> { arg::<PathBuf>(&args, key) };
    Ok(match method {
        // {} → versions and capabilities.
        "version" => json!({
            "jixel": env!("CARGO_PKG_VERSION"),
            "libjxl": jixel_core::libjxl_version(),
            "background_removal": background::is_available(),
            "decodable": Format::ALL.iter().filter(|f| f.can_decode()).collect::<Vec<_>>(),
            "encodable": Format::ALL.iter().filter(|f| f.can_encode()).collect::<Vec<_>>(),
        }),

        // {path} → Inspection.
        "inspect" => serde_json::to_value(inspect::inspect(&read(&path("path")?)?)?)?,

        // {path, max_side} → preview (buffer 0 = RGBA).
        "preview" => {
            let raster = decode_path(&path("path")?)?;
            out.preview(&raster, arg(&args, "max_side")?)?
        }

        // {input, output, options: ConvertOptions} → ConvertOutput.
        "convert" => {
            let opts: ConvertOptions = arg(&args, "options")?;
            let result = convert::convert(&read(&path("input")?)?, &opts)?;
            write(&path("output")?, &result.bytes)?;
            serde_json::to_value(&result)?
        }

        // {input, options: ConvertOptions, max_side} → in-memory encode for
        // the before/after view: {bytes, score, preview}. Nothing is written.
        "trial" => {
            let input = read(&path("input")?)?;
            let opts: ConvertOptions = arg(&args, "options")?;
            let original = decode_any(&input)?.raster;
            let result = convert::convert(&input, &opts)?;
            let decoded = decode_any(&result.bytes)?.raster;
            // Scores need the same geometry; resized trials are not scored.
            let score = (opts.resize.is_none())
                .then(|| {
                    let mut a = original.clone();
                    a.apply_orientation().ok()?;
                    let mut b = decoded.clone();
                    b.apply_orientation().ok()?;
                    compare::ssimulacra2(&a, &b).ok()
                })
                .flatten();
            json!({
                "result": result,
                "score": score,
                "preview": out.preview(&decoded, arg(&args, "max_side")?)?,
            })
        }

        // {input, output_dir, options: WebExportOptions, auto_focus} → Manifest.
        "web_export" => {
            let raster = decode_path(&path("input")?)?;
            let mut opts: WebExportOptions = arg(&args, "options")?;
            if opt::<bool>(&args, "auto_focus")?.unwrap_or(false) && opts.focus.is_none() {
                opts.focus = Some(subject::detect(&raster).focus);
            }
            let dir = path("output_dir")?;
            let export = web::export(&raster, &opts)?;
            for (p, bytes) in &export.files {
                write(&dir.join(p), bytes)?;
            }
            for (p, text) in &export.text_files {
                write(&dir.join(p), text.as_bytes())?;
            }
            let snippets: serde_json::Map<String, Value> = export
                .text_files
                .iter()
                .map(|(p, t)| (p.clone(), Value::String(t.clone())))
                .collect();
            json!({ "manifest": export.manifest, "snippets": snippets })
        }

        // {input, goal: BattleGoal, effort, output_dir?} → Battle.
        "battle" => {
            let input = path("input")?;
            let raster = decode_path(&input)?;
            let battle = compare::battle(&raster, arg(&args, "goal")?, arg(&args, "effort")?)?;
            if let Some(dir) = opt::<PathBuf>(&args, "output_dir")? {
                let stem = input
                    .file_stem()
                    .map_or("image".into(), |s| s.to_string_lossy().into_owned());
                for e in &battle.entries {
                    write(
                        &dir.join(format!("{stem}.{}", e.format.extension())),
                        &e.file,
                    )?;
                }
            }
            serde_json::to_value(&battle)?
        }

        // {reference, distorted} → {score}.
        "compare" => {
            let score = compare::ssimulacra2(
                &decode_path(&path("reference")?)?,
                &decode_path(&path("distorted")?)?,
            )?;
            json!({ "score": score })
        }

        // {path} → Subject.
        "detect_subject" => serde_json::to_value(subject::detect(&decode_path(&path("path")?)?))?,

        // {input, output, subject?: Subject, options: BoostOptions} → BoostResult.
        "subject_boost" => {
            let raster = decode_path(&path("input")?)?;
            let subject = match opt::<Subject>(&args, "subject")? {
                Some(s) => s,
                None => subject::detect(&raster),
            };
            let opts: BoostOptions = arg(&args, "options")?;
            let result = subject::encode_with_subject(&raster, &subject, &opts)?;
            write(&path("output")?, result.best())?;
            json!({ "subject": subject, "result": result })
        }

        // {input, output, format, options: BackgroundOptions, max_side} →
        // {bytes, preview}.
        "remove_background" => {
            let raster = decode_path(&path("input")?)?;
            let opts: BackgroundOptions = arg(&args, "options")?;
            let cut = background::remove_background(&raster, &opts)?;
            let format: Format = arg(&args, "format")?;
            let bytes = convert::encode_raster(
                &cut,
                &ConvertOptions {
                    quality: 100.0,
                    ..ConvertOptions::to(format)
                },
            )?;
            write(&path("output")?, &bytes)?;
            json!({ "bytes": bytes.len(), "preview": out.preview(&cut, arg(&args, "max_side")?)? })
        }

        // {input, output, rect: Rect, quality?} → {bytes, fidelity}.
        "crop" => {
            let mode = match opt::<f32>(&args, "quality")? {
                Some(quality) => CropMode::Reencode { quality },
                None => CropMode::PixelExact,
            };
            let (bytes, fidelity) = crop_file(
                &read(&path("input")?)?,
                arg(&args, "rect")?,
                mode,
                Default::default(),
            )?;
            write(&path("output")?, &bytes)?;
            json!({ "bytes": bytes.len(), "fidelity": fidelity })
        }

        // {input, output_dir, split: Split, format, quality} → {files}.
        "tiles" => {
            let input = path("input")?;
            let raster = decode_path(&input)?;
            let format: Format = arg(&args, "format")?;
            let opts = ConvertOptions {
                quality: arg(&args, "quality")?,
                ..ConvertOptions::to(format)
            };
            let stem = input
                .file_stem()
                .map_or("image".into(), |s| s.to_string_lossy().into_owned());
            let dir = path("output_dir")?;
            let split: Split = arg(&args, "split")?;
            let mut files = Vec::new();
            for t in tiles::split(&raster, split)? {
                let p = dir.join(format!("{stem}_{}_{}.{}", t.col, t.row, format.extension()));
                write(&p, &convert::encode_raster(&t.raster, &opts)?)?;
                files.push(json!({ "path": p, "col": t.col, "row": t.row, "rect": t.rect }));
            }
            json!({ "files": files })
        }

        // {input, output_dir, tile_size, overlap, format, quality} → {dzi, levels, tiles}.
        "deep_zoom" => {
            let input = path("input")?;
            let raster = decode_path(&input)?;
            let format: Format = arg(&args, "format")?;
            let opts = ConvertOptions {
                quality: arg(&args, "quality")?,
                ..ConvertOptions::to(format)
            };
            let stem = input
                .file_stem()
                .map_or("image".into(), |s| s.to_string_lossy().into_owned());
            let dir = path("output_dir")?;
            let dz = tiles::deep_zoom(
                &raster,
                &stem,
                arg(&args, "tile_size")?,
                arg(&args, "overlap")?,
                format.extension(),
                |t| convert::encode_raster(t, &opts),
            )?;
            let dzi = dir.join(format!("{stem}.dzi"));
            write(&dzi, dz.dzi.as_bytes())?;
            for (p, bytes) in &dz.files {
                write(&dir.join(p), bytes)?;
            }
            json!({ "dzi": dzi, "levels": dz.levels, "tiles": dz.files.len() })
        }

        // {input, output} → {bytes}. Restores the original JPEG.
        "reconstruct" => {
            let bytes = jxl::reconstruct_jpeg(&read(&path("input")?)?)?;
            write(&path("output")?, &bytes)?;
            json!({ "bytes": bytes.len() })
        }

        // {fractions, max_side} + input buffer 0 = a JPEG XL file →
        // [{fraction, bytes, preview?}].
        "progressive" => {
            let file = inputs
                .first()
                .ok_or("progressive needs the JPEG XL bytes as input 0")?;
            let max_side: u32 = arg(&args, "max_side")?;
            let steps = progressive::preview(file, &arg::<Vec<f32>>(&args, "fractions")?)?;
            let mut list = Vec::new();
            for s in steps {
                let preview = match &s.raster {
                    Some(r) => out.preview(r, max_side)?,
                    None => Value::Null,
                };
                list.push(json!({ "fraction": s.fraction, "bytes": s.bytes, "preview": preview }));
            }
            Value::Array(list)
        }

        // {input, options: ConvertOptions} → JPEG XL bytes in buffer 0, for
        // the progressive viewer (encode in memory, then call "progressive").
        "encode_jxl" => {
            let opts: ConvertOptions = arg(&args, "options")?;
            let result = convert::convert(
                &read(&path("input")?)?,
                &ConvertOptions {
                    target: Format::Jxl,
                    ..opts
                },
            )?;
            let buffer = out.push(result.bytes.clone());
            json!({ "buffer": buffer, "result": result })
        }

        other => return Err(format!("unknown method '{other}'").into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(method: &str, args: Value, inputs: &[&[u8]]) -> (Value, Vec<Vec<u8>>) {
        let m = CString::new(method).unwrap();
        let a = CString::new(args.to_string()).unwrap();
        let bufs: Vec<JixelBuffer> = inputs
            .iter()
            .map(|b| JixelBuffer {
                ptr: b.as_ptr() as *mut u8,
                len: b.len(),
            })
            .collect();
        unsafe {
            let r = jixel_call(m.as_ptr(), a.as_ptr(), bufs.as_ptr(), bufs.len());
            let json: Value =
                serde_json::from_str(CStr::from_ptr((*r).json).to_str().unwrap()).unwrap();
            let out = std::slice::from_raw_parts((*r).buffers, (*r).n_buffers)
                .iter()
                .map(|b| std::slice::from_raw_parts(b.ptr, b.len).to_vec())
                .collect();
            jixel_free(r);
            (json, out)
        }
    }

    fn sample_png(dir: &Path) -> PathBuf {
        let (w, h) = (96u32, 64u32);
        let data = (0..w * h)
            .flat_map(|i| [(i % w * 2) as u8, (i / w * 3) as u8, 128])
            .collect();
        let raster = Raster::from_u8(w, h, 3, data);
        let png = convert::encode_raster(&raster, &ConvertOptions::to(Format::Png)).unwrap();
        std::fs::create_dir_all(dir).unwrap();
        let p = dir.join("in.png");
        std::fs::write(&p, png).unwrap();
        p
    }

    #[test]
    fn version_and_unknown_method() {
        let (v, _) = call("version", json!({}), &[]);
        assert!(v["ok"]["libjxl"].as_str().unwrap().starts_with("0."));
        let (e, _) = call("nope", json!({}), &[]);
        assert!(e["error"].as_str().unwrap().contains("unknown method"));
    }

    #[test]
    fn convert_and_preview_through_the_abi() {
        let dir = std::env::temp_dir().join(format!("jixel-bridge-{}", std::process::id()));
        let input = sample_png(&dir);
        let output = dir.join("out.jxl");
        let (v, _) = call(
            "convert",
            json!({ "input": input, "output": output, "options": { "target": "jxl", "quality": 100 } }),
            &[],
        );
        assert_eq!(v["ok"]["fidelity"], "pixel_exact", "{v}");
        let (p, bufs) = call("preview", json!({ "path": output, "max_side": 48 }), &[]);
        assert_eq!(
            (p["ok"]["width"].as_u64(), p["ok"]["height"].as_u64()),
            (Some(48), Some(32))
        );
        assert_eq!(bufs[0].len(), 48 * 32 * 4);
        let jxl_bytes = std::fs::read(&output).unwrap();
        let (steps, _) = call(
            "progressive",
            json!({ "fractions": [1.0], "max_side": 64 }),
            &[&jxl_bytes],
        );
        assert_eq!(steps["ok"][0]["preview"]["width"], 64);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn errors_are_reported_not_thrown() {
        let (v, _) = call("inspect", json!({ "path": "/definitely/missing.png" }), &[]);
        assert!(v["error"].as_str().unwrap().contains("cannot read"));
    }
}
