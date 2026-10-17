//! `jixel`: every Jixel feature from the command line.
//!
//! Designed for scripts and build pipelines as much as for people: every
//! command accepts `--json` and then prints one machine-readable JSON value
//! per result on stdout.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use rayon::prelude::*;
use serde_json::json;

use jixel_core::analyze::{compare, inspect, progressive};
use jixel_core::codec::jxl::{self, FocusPoint};
use jixel_core::convert::{self, decode_any, ConvertOptions, Fidelity};
use jixel_core::metadata::MetadataPolicy;
use jixel_core::ops::background::{self, BackgroundOptions};
use jixel_core::ops::crop::{crop_file, CropMode};
use jixel_core::ops::resize::Fit;
use jixel_core::ops::subject::{self, BoostOptions, Subject};
use jixel_core::ops::tiles::{self, Split};
use jixel_core::ops::Rect;
use jixel_core::web::{self, WebExportOptions};
use jixel_core::Format;

#[derive(Parser)]
#[command(
    name = "jixel",
    version,
    about = "Convert, optimise and inspect images with JPEG XL.",
    long_about = "Convert, optimise and inspect images with JPEG XL.\n\n\
                  Examples:\n  \
                  jixel convert photo.jpg                 # lossless, ~20% smaller, reversible\n  \
                  jixel convert *.png -q 90 -o out/       # batch, visually lossless\n  \
                  jixel web hero.jpg --hero -o public/images/hero\n  \
                  jixel battle photo.png                  # JPEG XL vs AVIF vs WebP vs JPEG"
)]
struct Cli {
    /// Print machine-readable JSON instead of text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert images between formats (default: to JPEG XL).
    Convert(ConvertArgs),
    /// Create responsive web versions plus HTML/Next.js/WordPress/server snippets.
    Web(WebArgs),
    /// Show what is inside image files, with suggestions.
    Info { inputs: Vec<PathBuf> },
    /// Score how close an image is to a reference (SSIMULACRA2, 0–100).
    Compare {
        reference: PathBuf,
        distorted: PathBuf,
    },
    /// Encode one image as JPEG XL, AVIF, WebP and JPEG and compare them.
    Battle(BattleArgs),
    /// Crop to JPEG XL, pixel-exact by default.
    Crop(CropArgs),
    /// Split an image into a grid of tiles.
    Tiles(TilesArgs),
    /// Build a Deep Zoom (DZI) pyramid for zoomable viewers.
    Dzi(DziArgs),
    /// Remove the background (on-device AI).
    Rmbg(RmbgArgs),
    /// Give the main subject more quality than the background.
    Subject(SubjectArgs),
    /// Restore the original JPEG from a recompressed JPEG XL.
    Reconstruct {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[command(flatten)]
        write: WriteArgs,
    },
    /// Save what a progressive JPEG XL looks like while it downloads.
    Preview(PreviewArgs),
}

#[derive(Args)]
struct WriteArgs {
    /// Replace existing files.
    #[arg(long)]
    overwrite: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum MetadataArg {
    Keep,
    StripGps,
    StripAll,
}

impl From<MetadataArg> for MetadataPolicy {
    fn from(m: MetadataArg) -> Self {
        match m {
            MetadataArg::Keep => MetadataPolicy::Keep,
            MetadataArg::StripGps => MetadataPolicy::StripGps,
            MetadataArg::StripAll => MetadataPolicy::StripAll,
        }
    }
}

#[derive(Args)]
struct ConvertArgs {
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    /// Output format (jxl, avif, webp, jpg, png, gif, tiff, …). Default:
    /// taken from the --output extension, otherwise jxl.
    #[arg(short, long, value_parser = parse_format)]
    format: Option<Format>,
    /// Output file (one input) or folder (several inputs).
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Quality 0–100; 100 = lossless. JPEG → JPEG XL ignores it unless
    /// --reencode-jpeg is given (lossless recompression is used instead).
    #[arg(short, long, default_value_t = 90.0)]
    quality: f32,
    /// Encoder effort 1–10: higher is smaller but slower.
    #[arg(short, long, default_value_t = 7)]
    effort: u8,
    /// Re-encode JPEGs instead of recompressing them losslessly.
    #[arg(long)]
    reencode_jpeg: bool,
    #[arg(long)]
    max_width: Option<u32>,
    #[arg(long)]
    max_height: Option<u32>,
    /// Progressive JPEG XL (a preview appears early in the download).
    #[arg(long)]
    progressive: bool,
    /// Load this point first: "x,y" in pixels.
    #[arg(long, value_parser = parse_point, conflicts_with = "auto_focus")]
    focus: Option<FocusPoint>,
    /// Detect the main subject and load it first.
    #[arg(long)]
    auto_focus: bool,
    #[arg(long, value_enum, default_value = "strip-gps")]
    metadata: MetadataArg,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct WebArgs {
    input: PathBuf,
    /// Output folder.
    #[arg(short, long)]
    output: PathBuf,
    /// File name stem (default: input file name).
    #[arg(long)]
    name: Option<String>,
    /// Comma-separated widths.
    #[arg(long, value_delimiter = ',', default_values_t = web::DEFAULT_WIDTHS)]
    widths: Vec<u32>,
    #[arg(short, long, default_value_t = 85.0)]
    quality: f32,
    /// Comma-separated fallback formats, best first.
    #[arg(long, value_delimiter = ',', default_value = "avif,jpg", value_parser = parse_format)]
    fallbacks: Vec<Format>,
    #[arg(short, long, default_value_t = 7)]
    effort: u8,
    /// URL path the files will be served from.
    #[arg(long, default_value = "/images/")]
    base_url: String,
    /// The `sizes` attribute.
    #[arg(long, default_value = "100vw")]
    sizes: String,
    #[arg(long, default_value = "")]
    alt: String,
    /// Above-the-fold image: eager loading with high priority.
    #[arg(long)]
    hero: bool,
    /// Detect the main subject and load it first.
    #[arg(long)]
    auto_focus: bool,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct BattleArgs {
    input: PathBuf,
    /// Target SSIMULACRA2 score each format must reach (fair comparison).
    #[arg(long, default_value_t = 80.0, conflicts_with = "quality")]
    score: f64,
    /// Use the same quality setting for all formats instead.
    #[arg(short, long)]
    quality: Option<f32>,
    #[arg(short, long, default_value_t = 7)]
    effort: u8,
    /// Also save each encoded file in this folder.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct CropArgs {
    input: PathBuf,
    /// "x,y,width,height" in pixels.
    #[arg(long, value_parser = parse_rect)]
    rect: Rect,
    /// Re-encode at this quality instead of storing exact pixels.
    #[arg(short, long)]
    quality: Option<f32>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "strip-gps")]
    metadata: MetadataArg,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct TilesArgs {
    input: PathBuf,
    /// Grid as "COLSxROWS", e.g. 3x3.
    #[arg(long, value_parser = parse_size, conflicts_with = "size", required_unless_present = "size")]
    grid: Option<(u32, u32)>,
    /// Tile size as "WIDTHxHEIGHT", e.g. 512x512.
    #[arg(long, value_parser = parse_size)]
    size: Option<(u32, u32)>,
    #[arg(short, long, default_value = "jxl", value_parser = parse_format)]
    format: Format,
    /// Quality 0–100 (100 = lossless, the default: tiles are pixel-exact).
    #[arg(short, long, default_value_t = 100.0)]
    quality: f32,
    #[arg(short, long)]
    output: PathBuf,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct DziArgs {
    input: PathBuf,
    #[arg(long, default_value_t = 254)]
    tile_size: u32,
    #[arg(long, default_value_t = 1)]
    overlap: u32,
    #[arg(short, long, default_value = "jxl", value_parser = parse_format)]
    format: Format,
    #[arg(short, long, default_value_t = 85.0)]
    quality: f32,
    #[arg(short, long)]
    output: PathBuf,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct RmbgArgs {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Output format; needs transparency unless --fill is used.
    #[arg(short, long, default_value = "png", value_parser = parse_format)]
    format: Format,
    /// Background colour instead of transparency, e.g. "#ffffff".
    #[arg(long, value_parser = parse_color)]
    fill: Option<[u8; 3]>,
    /// Hard cut-out edges instead of soft ones.
    #[arg(long)]
    hard: bool,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct SubjectArgs {
    input: PathBuf,
    /// Subject as "x,y,width,height" (default: detected automatically).
    #[arg(long, value_parser = parse_rect)]
    rect: Option<Rect>,
    #[arg(short, long, default_value_t = 85.0)]
    quality: f32,
    #[arg(short, long, default_value_t = 7)]
    effort: u8,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[command(flatten)]
    write: WriteArgs,
}

#[derive(Args)]
struct PreviewArgs {
    input: PathBuf,
    /// Comma-separated fractions of the file, e.g. 0.05,0.15,0.3.
    #[arg(long, value_delimiter = ',', default_values_t = progressive::DEFAULT_STEPS)]
    steps: Vec<f32>,
    #[arg(short, long)]
    output: PathBuf,
    #[command(flatten)]
    write: WriteArgs,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        if cli.json {
            println!("{}", json!({ "error": format!("{e:#}") }));
        } else {
            eprintln!("error: {e:#}");
        }
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<()> {
    let out = Output { json: cli.json };
    match &cli.command {
        Command::Convert(a) => cmd_convert(a, &out),
        Command::Web(a) => cmd_web(a, &out),
        Command::Info { inputs } => {
            for path in inputs {
                let i =
                    inspect::inspect(&read(path)?).with_context(|| path.display().to_string())?;
                out.result(&i, || {
                    let mut s = format!(
                        "{}\n  {} · {}×{} · {} bit{} · {}",
                        path.display(),
                        i.format.label(),
                        i.width,
                        i.height,
                        i.bits_per_sample,
                        if i.has_alpha { " · alpha" } else { "" },
                        human_bytes(i.file_bytes)
                    );
                    s += &format!(" · {:.2} bits/pixel", i.bits_per_pixel);
                    if i.frames > 1 {
                        s += &format!(" · {} frames", i.frames);
                    }
                    if let Some(p) = &i.jxl {
                        s += &format!(
                            "\n  JPEG XL: {}{}{}",
                            if p.uses_original_profile {
                                "lossless/original colour"
                            } else {
                                "lossy (XYB)"
                            },
                            if p.jpeg_reconstruction {
                                " · contains original JPEG"
                            } else {
                                ""
                            },
                            if p.is_hdr() { " · HDR" } else { "" }
                        );
                    }
                    for tip in &i.suggestions {
                        s += &format!("\n  • {tip}");
                    }
                    s
                });
            }
            Ok(())
        }
        Command::Compare {
            reference,
            distorted,
        } => {
            let a = decode_any(&read(reference)?)?.raster;
            let b = decode_any(&read(distorted)?)?.raster;
            let score = compare::ssimulacra2(&a, &b)?;
            out.result(&json!({ "ssimulacra2": score }), || {
                format!("SSIMULACRA2: {score:.2} ({})", describe_score(score))
            });
            Ok(())
        }
        Command::Battle(a) => cmd_battle(a, &out),
        Command::Crop(a) => {
            let mode = match a.quality {
                Some(quality) => CropMode::Reencode { quality },
                None => CropMode::PixelExact,
            };
            let (bytes, fidelity) = crop_file(&read(&a.input)?, a.rect, mode, a.metadata.into())?;
            let path = a
                .output
                .clone()
                .unwrap_or_else(|| sibling(&a.input, "crop", "jxl"));
            write(&path, &bytes, &a.write)?;
            out.result(
                &json!({ "output": path, "bytes": bytes.len(), "fidelity": fidelity }),
                || {
                    format!(
                        "{} ({}, {})",
                        path.display(),
                        human_bytes(bytes.len()),
                        fidelity_text(fidelity)
                    )
                },
            );
            Ok(())
        }
        Command::Tiles(a) => {
            let raster = decode_any(&read(&a.input)?)?.raster;
            let how = match (a.grid, a.size) {
                (Some((cols, rows)), _) => Split::Grid { cols, rows },
                (_, Some((width, height))) => Split::Size { width, height },
                _ => unreachable!("clap requires one of them"),
            };
            let opts = ConvertOptions {
                quality: a.quality,
                ..ConvertOptions::to(a.format)
            };
            let tiles = tiles::split(&raster, how)?;
            let files: Vec<(PathBuf, Vec<u8>)> = tiles
                .par_iter()
                .map(|t| {
                    let name = format!(
                        "{}_{}_{}.{}",
                        stem(&a.input),
                        t.col,
                        t.row,
                        a.format.extension()
                    );
                    Ok((
                        a.output.join(name),
                        convert::encode_raster(&t.raster, &opts)?,
                    ))
                })
                .collect::<Result<_>>()?;
            for (path, bytes) in &files {
                write(path, bytes, &a.write)?;
            }
            out.result(&json!({ "tiles": files.len(), "folder": a.output }), || {
                format!("{} tiles written to {}", files.len(), a.output.display())
            });
            Ok(())
        }
        Command::Dzi(a) => {
            let raster = decode_any(&read(&a.input)?)?.raster;
            let opts = ConvertOptions {
                quality: a.quality,
                ..ConvertOptions::to(a.format)
            };
            let name = stem(&a.input);
            let dz = tiles::deep_zoom(
                &raster,
                &name,
                a.tile_size,
                a.overlap,
                a.format.extension(),
                |t| convert::encode_raster(t, &opts),
            )?;
            write(
                &a.output.join(format!("{name}.dzi")),
                dz.dzi.as_bytes(),
                &a.write,
            )?;
            for (path, bytes) in &dz.files {
                write(&a.output.join(path), bytes, &a.write)?;
            }
            out.result(
                &json!({ "levels": dz.levels, "tiles": dz.files.len(), "folder": a.output }),
                || {
                    format!(
                        "Deep Zoom: {} levels, {} tiles in {}",
                        dz.levels,
                        dz.files.len(),
                        a.output.display()
                    )
                },
            );
            Ok(())
        }
        Command::Rmbg(a) => {
            if !background::is_available() {
                bail!("this build of jixel does not include background removal");
            }
            if a.fill.is_none() && !a.format.supports_alpha() {
                bail!(
                    "{} has no transparency: use --fill or a format like png, webp or jxl",
                    a.format
                );
            }
            let raster = decode_any(&read(&a.input)?)?.raster;
            let cut = background::remove_background(
                &raster,
                &BackgroundOptions {
                    refine_edges: true,
                    hard_edges: a.hard,
                    fill: a.fill,
                },
            )?;
            let opts = ConvertOptions {
                quality: 100.0,
                ..ConvertOptions::to(a.format)
            };
            let bytes = convert::encode_raster(&cut, &opts)?;
            let path = a
                .output
                .clone()
                .unwrap_or_else(|| sibling(&a.input, "cutout", a.format.extension()));
            write(&path, &bytes, &a.write)?;
            out.result(&json!({ "output": path, "bytes": bytes.len() }), || {
                format!("{} ({})", path.display(), human_bytes(bytes.len()))
            });
            Ok(())
        }
        Command::Subject(a) => cmd_subject(a, &out),
        Command::Reconstruct {
            input,
            output,
            write: w,
        } => {
            let bytes = jxl::reconstruct_jpeg(&read(input)?)?;
            let path = output
                .clone()
                .unwrap_or_else(|| input.with_extension("jpg"));
            write(&path, &bytes, w)?;
            out.result(&json!({ "output": path, "bytes": bytes.len() }), || {
                format!(
                    "{} restored bit for bit ({})",
                    path.display(),
                    human_bytes(bytes.len())
                )
            });
            Ok(())
        }
        Command::Preview(a) => {
            let bytes = read(&a.input)?;
            let steps = progressive::preview(&bytes, &a.steps)?;
            let mut written = Vec::new();
            for s in &steps {
                let Some(r) = &s.raster else { continue };
                let png = convert::encode_raster(
                    r,
                    &ConvertOptions {
                        quality: 100.0,
                        ..ConvertOptions::to(Format::Png)
                    },
                )?;
                let path = a.output.join(format!(
                    "{}_{:03}pct.png",
                    stem(&a.input),
                    (s.fraction * 100.0).round() as u32
                ));
                write(&path, &png, &a.write)?;
                written.push(json!({ "fraction": s.fraction, "bytes": s.bytes, "output": path }));
            }
            out.result(&json!(written), || {
                let shown: Vec<String> = steps
                    .iter()
                    .map(|s| {
                        format!(
                            "{:.0}%: {}",
                            s.fraction * 100.0,
                            if s.raster.is_some() {
                                "visible"
                            } else {
                                "nothing yet"
                            }
                        )
                    })
                    .collect();
                format!("{}\nSaved to {}", shown.join("\n"), a.output.display())
            });
            Ok(())
        }
    }
}

fn cmd_convert(a: &ConvertArgs, out: &Output) -> Result<()> {
    let to_folder = a.inputs.len() > 1 || a.output.as_ref().is_some_and(|p| p.is_dir());
    let format = a
        .format
        .or_else(|| {
            let ext = a
                .output
                .as_ref()
                .filter(|_| !to_folder)?
                .extension()?
                .to_str()?;
            parse_format(ext).ok()
        })
        .unwrap_or(Format::Jxl);
    let fit = (a.max_width.is_some() || a.max_height.is_some()).then_some(Fit {
        max_width: a.max_width,
        max_height: a.max_height,
    });
    let results: Vec<Result<serde_json::Value>> = a
        .inputs
        .par_iter()
        .map(|input| -> Result<serde_json::Value> {
            let bytes = read(input)?;
            let mut focus = a.focus;
            if a.auto_focus {
                focus = Some(subject::detect(&decode_any(&bytes)?.raster).focus);
            }
            let opts = ConvertOptions {
                quality: a.quality,
                effort: a.effort,
                jpeg_recompression: !a.reencode_jpeg,
                progressive: a.progressive,
                focus,
                resize: fit,
                metadata: a.metadata.into(),
                ..ConvertOptions::to(format)
            };
            let result =
                convert::convert(&bytes, &opts).with_context(|| input.display().to_string())?;
            let path = match &a.output {
                Some(dir) if to_folder => {
                    dir.join(format!("{}.{}", stem(input), format.extension()))
                }
                Some(file) => file.clone(),
                None => input.with_extension(format.extension()),
            };
            if path == *input {
                bail!(
                    "{}: output would replace the input; use -o",
                    input.display()
                );
            }
            write(&path, &result.bytes, &a.write)?;
            let line = format!(
                "{} → {}  {} → {} ({:+.0}%) · {}",
                input.display(),
                path.display(),
                human_bytes(result.input_bytes),
                human_bytes(result.output_bytes),
                result.saving_percent(),
                fidelity_text(result.fidelity)
            );
            out.text(|| {
                let mut s = line.clone();
                for n in &result.notes {
                    s += &format!("\n  note: {n}");
                }
                s
            });
            Ok(json!({ "input": input, "output": path, "result": result }))
        })
        .collect();
    let mut failed = 0;
    for r in results {
        match r {
            Ok(v) => out.json_only(&v),
            Err(e) => {
                failed += 1;
                out.error(&e);
            }
        }
    }
    if failed > 0 {
        bail!("{failed} of {} files failed", a.inputs.len());
    }
    Ok(())
}

fn cmd_web(a: &WebArgs, out: &Output) -> Result<()> {
    let raster = decode_any(&read(&a.input)?)?.raster;
    let focus = a.auto_focus.then(|| subject::detect(&raster).focus);
    let opts = WebExportOptions {
        name: a.name.clone().unwrap_or_else(|| slug(&stem(&a.input))),
        widths: a.widths.clone(),
        quality: a.quality,
        fallbacks: a.fallbacks.clone(),
        effort: a.effort,
        focus,
        base_url: a.base_url.clone(),
        sizes: a.sizes.clone(),
        alt: a.alt.clone(),
        hero: a.hero,
        ..Default::default()
    };
    let export = web::export(&raster, &opts)?;
    for (path, bytes) in &export.files {
        write(&a.output.join(path), bytes, &a.write)?;
    }
    for (path, text) in &export.text_files {
        write(&a.output.join(path), text.as_bytes(), &a.write)?;
    }
    let m = &export.manifest;
    out.result(m, || {
        let mut s = format!(
            "Exported {} files to {}\n",
            export.files.len(),
            a.output.display()
        );
        for (format, total) in &m.totals {
            s += &format!("  {:<8} {:>10}", format.label(), human_bytes(*total));
            if *format != Format::Jxl {
                if let Some(p) = m.jxl_saving_vs(*format) {
                    s += &format!(
                        "   JPEG XL is {:.0}% {}",
                        p.abs(),
                        if p <= 0.0 {
                            "smaller"
                        } else {
                            "larger (but scored at the same setting, not the same quality)"
                        }
                    );
                }
            }
            s += "\n";
        }
        s += &format!(
            "Paste {}:\n\n{}",
            "snippets/picture.html",
            web::snippets::picture_html(m)
        );
        s
    });
    Ok(())
}

fn cmd_battle(a: &BattleArgs, out: &Output) -> Result<()> {
    let raster = decode_any(&read(&a.input)?)?.raster;
    let goal = match a.quality {
        Some(quality) => compare::BattleGoal::SameSetting { quality },
        None => compare::BattleGoal::SameScore { score: a.score },
    };
    let battle = compare::battle(&raster, goal, a.effort)?;
    if let Some(dir) = &a.output {
        for e in &battle.entries {
            write(
                &dir.join(format!("{}.{}", stem(&a.input), e.format.extension())),
                &e.file,
                &WriteArgs { overwrite: true },
            )?;
        }
    }
    out.result(&battle, || {
        let mut s = format!("{}×{}, {:?}\n", battle.width, battle.height, battle.goal);
        for e in &battle.entries {
            s += &format!(
                "  {:<8} {:>10}  {:>5.2} bpp  score {:>6}  q{:<3}{}{}\n",
                e.format.label(),
                human_bytes(e.bytes),
                e.bpp,
                e.score.map_or("—".into(), |v| format!("{v:.1}")),
                e.quality,
                e.vs_jpeg_percent
                    .filter(|_| e.format != Format::Jpeg)
                    .map_or(String::new(), |p| format!("  {p:+.0}% vs JPEG")),
                e.note
                    .as_ref()
                    .map_or(String::new(), |n| format!("  ({n})")),
            );
        }
        s
    });
    Ok(())
}

fn cmd_subject(a: &SubjectArgs, out: &Output) -> Result<()> {
    let raster = decode_any(&read(&a.input)?)?.raster;
    let (w, h) = if raster.orientation > 4 {
        (raster.height, raster.width)
    } else {
        (raster.width, raster.height)
    };
    let subj = match a.rect {
        Some(rect) => Subject::manual(rect, w, h)?,
        None => subject::detect(&raster),
    };
    let r = subject::encode_with_subject(
        &raster,
        &subj,
        &BoostOptions {
            quality: a.quality,
            effort: a.effort,
            ..Default::default()
        },
    )?;
    let path = a
        .output
        .clone()
        .unwrap_or_else(|| sibling(&a.input, "subject", "jxl"));
    write(&path, r.best(), &a.write)?;
    out.result(&json!({ "output": path, "subject": subj, "result": r }), || {
        let b = &subj.bbox;
        let mut s = format!(
            "Subject ({:?}): {}×{} at {},{} — loads first\n  single layer: {} · subject score {:.1}\n",
            subj.source,
            b.width,
            b.height,
            b.x,
            b.y,
            human_bytes(r.single_bytes),
            r.single_subject_score
        );
        match (r.boosted_bytes, r.boosted_subject_score) {
            (Some(bytes), Some(score)) => {
                s += &format!("  subject boost: {} · subject score {score:.1}\n", human_bytes(bytes));
                s += if r.recommend_boost { "  → boost used" } else { "  → single layer used (boost did not pay off)" };
            }
            _ => s += "  subject covers most of the image: boost not needed",
        }
        s += &format!("\n{}", path.display());
        s
    });
    Ok(())
}

// ---------------------------------------------------------------- output --

struct Output {
    json: bool,
}

impl Output {
    /// Prints `value` as JSON, or the text produced by `text`.
    fn result<T: serde::Serialize>(&self, value: &T, text: impl FnOnce() -> String) {
        if self.json {
            println!("{}", serde_json::to_string(value).unwrap_or_default());
        } else {
            println!("{}", text());
        }
    }

    fn text(&self, text: impl FnOnce() -> String) {
        if !self.json {
            println!("{}", text());
        }
    }

    fn json_only(&self, value: &serde_json::Value) {
        if self.json {
            println!("{value}");
        }
    }

    fn error(&self, e: &anyhow::Error) {
        if self.json {
            println!("{}", json!({ "error": format!("{e:#}") }));
        } else {
            eprintln!("error: {e:#}");
        }
    }
}

fn human_bytes(n: usize) -> String {
    match n {
        n if n < 1024 => format!("{n} B"),
        n if n < 1024 * 1024 => format!("{:.1} KB", n as f64 / 1024.0),
        n => format!("{:.2} MB", n as f64 / 1024.0 / 1024.0),
    }
}

fn fidelity_text(f: Fidelity) -> &'static str {
    match f {
        Fidelity::Reversible => "lossless, original restorable",
        Fidelity::PixelExact => "lossless",
        Fidelity::Lossy => "lossy",
    }
}

fn describe_score(score: f64) -> &'static str {
    match score {
        s if s >= 90.0 => "visually lossless",
        s if s >= 80.0 => "very high quality",
        s if s >= 70.0 => "high quality",
        s if s >= 50.0 => "medium quality",
        _ => "low quality",
    }
}

// ----------------------------------------------------------------- files --

fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))
}

fn write(path: &Path, bytes: &[u8], args: &WriteArgs) -> Result<()> {
    if path.exists() && !args.overwrite {
        bail!("{} already exists (use --overwrite)", path.display());
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    std::fs::write(path, bytes).with_context(|| format!("cannot write {}", path.display()))
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .map_or("image".into(), |s| s.to_string_lossy().into_owned())
}

/// `photo.jpg` + ("crop", "jxl") → `photo-crop.jxl`.
fn sibling(path: &Path, suffix: &str, ext: &str) -> PathBuf {
    path.with_file_name(format!("{}-{suffix}.{ext}", stem(path)))
}

/// A URL-friendly file name: lowercase ASCII, digits and dashes.
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "image".into()
    } else {
        out
    }
}

// --------------------------------------------------------------- parsing --

fn parse_format(s: &str) -> Result<Format, String> {
    Format::from_extension(s.trim_start_matches('.'))
        .filter(|f| f.can_encode())
        .ok_or_else(|| format!("unknown or unsupported output format '{s}'"))
}

fn numbers(s: &str, sep: char, n: usize) -> Result<Vec<u32>, String> {
    let v: Vec<u32> = s
        .split(sep)
        .map(|p| p.trim().parse::<u32>())
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    if v.len() != n {
        return Err(format!("expected {n} numbers separated by '{sep}'"));
    }
    Ok(v)
}

fn parse_rect(s: &str) -> Result<Rect, String> {
    let v = numbers(s, ',', 4)?;
    Ok(Rect::new(v[0], v[1], v[2], v[3]))
}

fn parse_point(s: &str) -> Result<FocusPoint, String> {
    let v = numbers(s, ',', 2)?;
    Ok(FocusPoint { x: v[0], y: v[1] })
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let v = numbers(&s.to_ascii_lowercase(), 'x', 2)?;
    Ok((v[0], v[1]))
}

fn parse_color(s: &str) -> Result<[u8; 3], String> {
    let hex = s.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("expected a colour like #ffffff".into());
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string());
    Ok([byte(0)?, byte(2)?, byte(4)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn parsers() {
        assert_eq!(parse_rect("1, 2,3,4").unwrap(), Rect::new(1, 2, 3, 4));
        assert_eq!(parse_size("3X2").unwrap(), (3, 2));
        assert_eq!(parse_color("#ff8000").unwrap(), [255, 128, 0]);
        assert_eq!(parse_format("jpg").unwrap(), Format::Jpeg);
        assert!(parse_format("heic").is_err());
        assert_eq!(slug("My Photo (1)"), "my-photo-1");
    }
}
