//! JPEG XL encoding: still images, layered images, animations, and lossless
//! JPEG recompression.

use std::mem::MaybeUninit;

use jixel_libjxl_sys as sys;

use super::{box_type, JxlEncodeOptions, JxlQuality, Runner};
use crate::metadata::{strip_gps_in_jpeg, Metadata, MetadataPolicy};
use crate::{Error, Raster, Result, Samples};

/// Encodes one still image.
pub fn encode(raster: &Raster, opts: &JxlEncodeOptions) -> Result<Vec<u8>> {
    let mut enc = Encoder::new()?;
    enc.set_basic_info(raster, opts.quality, None)?;
    enc.set_color(raster, opts.quality)?;
    enc.add_metadata(&raster.metadata.with_policy(opts.metadata))?;
    let settings = enc.frame_settings(opts, opts.quality)?;
    enc.add_frame(settings, raster, None)?;
    enc.finish()
}

/// A layer placed on top of the base image: used by *subject boost* to store
/// the subject at higher quality than the background.
#[derive(Clone, Debug)]
pub struct Layer {
    pub raster: Raster,
    /// Position of the layer's top-left corner on the canvas.
    pub x: i32,
    pub y: i32,
    /// Quality of this layer only.
    pub quality: JxlQuality,
}

/// Encodes `base` plus `layers`, composited by the decoder into one still
/// image. Every frame has duration 0 and replaces the pixels below it.
pub fn encode_layers(base: &Raster, layers: &[Layer], opts: &JxlEncodeOptions) -> Result<Vec<u8>> {
    let any_lossless = opts.quality == JxlQuality::Lossless
        || layers.iter().any(|l| l.quality == JxlQuality::Lossless);
    let profile_quality = if any_lossless {
        JxlQuality::Lossless
    } else {
        opts.quality
    };
    let mut enc = Encoder::new()?;
    enc.set_basic_info(base, profile_quality, None)?;
    enc.set_color(base, profile_quality)?;
    enc.add_metadata(&base.metadata.with_policy(opts.metadata))?;

    let settings = enc.frame_settings(opts, opts.quality)?;
    enc.add_frame(settings, base, None)?;
    for layer in layers {
        if layer.raster.channels != base.channels
            || layer.raster.samples.bits() != base.samples.bits()
        {
            return Err(Error::InvalidInput(
                "layers must have the same pixel layout as the base".into(),
            ));
        }
        // The focus point is in canvas coordinates; libjxl wants it relative
        // to the frame and rejects points outside it.
        let focus = opts.focus.and_then(|f| {
            let (fx, fy) = (f.x as i64 - layer.x as i64, f.y as i64 - layer.y as i64);
            let inside = (0..layer.raster.width as i64).contains(&fx)
                && (0..layer.raster.height as i64).contains(&fy);
            inside.then_some(super::FocusPoint {
                x: fx as u32,
                y: fy as u32,
            })
        });
        let settings = enc.frame_settings(
            &JxlEncodeOptions {
                focus,
                ..opts.clone()
            },
            layer.quality,
        )?;
        enc.add_frame(settings, &layer.raster, Some((layer.x, layer.y, 0)))?;
    }
    enc.finish()
}

/// Encodes an animation. `frames` are `(image, duration in milliseconds)`;
/// `loops` is the repeat count, 0 meaning forever.
pub fn encode_animation(
    frames: &[(Raster, u32)],
    loops: u32,
    opts: &JxlEncodeOptions,
) -> Result<Vec<u8>> {
    let (first, _) = frames
        .first()
        .ok_or_else(|| Error::InvalidInput("an animation needs frames".into()))?;
    let mut enc = Encoder::new()?;
    enc.set_basic_info(first, opts.quality, Some(loops))?;
    enc.set_color(first, opts.quality)?;
    enc.add_metadata(&first.metadata.with_policy(opts.metadata))?;
    let settings = enc.frame_settings(opts, opts.quality)?;
    for (raster, duration) in frames {
        if (raster.width, raster.height, raster.channels)
            != (first.width, first.height, first.channels)
        {
            return Err(Error::InvalidInput(
                "all animation frames must have the same size".into(),
            ));
        }
        enc.add_frame(settings, raster, Some((0, 0, (*duration).max(1))))?;
    }
    enc.finish()
}

/// **Lossless JPEG recompression.** Re-packs the JPEG's own DCT coefficients
/// with JPEG XL's better entropy coding: typically ~20% smaller, with zero
/// generation loss, and [`super::reconstruct_jpeg`] restores the original
/// file bit for bit.
///
/// Only `effort`, `focus` and `metadata` from `opts` apply. With
/// [`MetadataPolicy::StripGps`] the GPS data is zeroed *inside* the JPEG
/// first, so the restored JPEG is the original minus its location.
/// With [`MetadataPolicy::StripAll`] the restored JPEG has no metadata.
pub fn transcode_jpeg(jpeg: &[u8], opts: &JxlEncodeOptions) -> Result<Vec<u8>> {
    let mut input = jpeg.to_vec();
    if opts.metadata == MetadataPolicy::StripGps {
        strip_gps_in_jpeg(&mut input);
    }
    let keep = i64::from(opts.metadata != MetadataPolicy::StripAll);

    let enc = Encoder::new()?;
    // SAFETY: `enc.ptr` is a live encoder for every call below.
    unsafe {
        enc.check(
            sys::JxlEncoderUseContainer(enc.ptr, sys::JXL_TRUE),
            "enabling the container",
        )?;
        enc.check(
            sys::JxlEncoderStoreJPEGMetadata(enc.ptr, sys::JXL_TRUE),
            "storing JPEG data",
        )?;
    }
    let settings = enc.frame_settings(opts, JxlQuality::Lossless)?;
    // SAFETY: valid settings pointer owned by `enc`; `input` outlives the call.
    unsafe {
        for (option, value) in [
            (sys::JXL_ENC_FRAME_SETTING_JPEG_KEEP_EXIF, keep),
            (sys::JXL_ENC_FRAME_SETTING_JPEG_KEEP_XMP, keep),
            (sys::JXL_ENC_FRAME_SETTING_JPEG_KEEP_JUMBF, keep),
        ] {
            enc.check(
                sys::JxlEncoderFrameSettingsSetOption(settings, option, value),
                "setting JPEG metadata options",
            )?;
        }
        let status = sys::JxlEncoderAddJPEGFrame(settings, input.as_ptr(), input.len());
        if status != sys::JXL_ENC_SUCCESS {
            // The usual cause is a JPEG libjxl cannot represent exactly
            // (e.g. arithmetic coding, or a broken file).
            return Err(Error::Encode(
                "this JPEG cannot be recompressed losslessly; convert it with a quality setting instead".into(),
            ));
        }
    }
    enc.finish()
}

/// Owns a `JxlEncoder` and its thread runner.
struct Encoder {
    ptr: *mut sys::JxlEncoder,
    _runner: Runner,
}

impl Encoder {
    fn new() -> Result<Self> {
        // SAFETY: null memory manager = default allocator.
        let ptr = unsafe { sys::JxlEncoderCreate(std::ptr::null()) };
        if ptr.is_null() {
            return Err(Error::Encode("out of memory".into()));
        }
        let runner = Runner::new();
        let enc = Encoder {
            ptr,
            _runner: runner,
        };
        // SAFETY: both pointers are live; the runner outlives the encoder
        // because fields drop after `Drop::drop` runs.
        unsafe {
            if !enc._runner.as_ptr().is_null() {
                enc.check(
                    sys::JxlEncoderSetParallelRunner(
                        ptr,
                        sys::JxlThreadParallelRunner,
                        enc._runner.as_ptr(),
                    ),
                    "starting worker threads",
                )?;
            }
            // Needed to convert ICC-profiled input to XYB for lossy encoding.
            sys::JxlEncoderSetCms(ptr, *sys::JxlGetDefaultCms());
        }
        Ok(enc)
    }

    /// Turns a libjxl status into a `Result`, with libjxl's error code.
    fn check(&self, status: sys::JxlEncoderStatus, context: &'static str) -> Result<()> {
        if status == sys::JXL_ENC_SUCCESS {
            Ok(())
        } else {
            // SAFETY: live encoder.
            let code = unsafe { sys::JxlEncoderGetError(self.ptr) };
            Err(Error::Libjxl { context, code })
        }
    }

    fn set_basic_info(
        &mut self,
        raster: &Raster,
        quality: JxlQuality,
        animation_loops: Option<u32>,
    ) -> Result<()> {
        // SAFETY: Init fully initialises the struct.
        let mut info = unsafe {
            let mut info = MaybeUninit::uninit();
            sys::JxlEncoderInitBasicInfo(info.as_mut_ptr());
            info.assume_init()
        };
        info.xsize = raster.width;
        info.ysize = raster.height;
        let (bits, exponent) = sample_bits(raster);
        info.bits_per_sample = bits;
        info.exponent_bits_per_sample = exponent;
        info.num_color_channels = if raster.is_gray() { 1 } else { 3 };
        if raster.has_alpha() {
            info.num_extra_channels = 1;
            info.alpha_bits = bits;
            info.alpha_exponent_bits = exponent;
        }
        // Lossless must keep the original colour space; lossy uses XYB.
        info.uses_original_profile = i32::from(quality == JxlQuality::Lossless);
        info.orientation = raster.orientation.clamp(1, 8).into();
        if let Some(loops) = animation_loops {
            info.have_animation = sys::JXL_TRUE;
            // Ticks are milliseconds.
            info.animation.tps_numerator = 1000;
            info.animation.tps_denominator = 1;
            info.animation.num_loops = loops;
        }
        // SAFETY: live encoder, valid struct.
        unsafe {
            self.check(
                sys::JxlEncoderSetBasicInfo(self.ptr, &info),
                "setting image properties",
            )
        }
    }

    fn set_color(&mut self, raster: &Raster, quality: JxlQuality) -> Result<()> {
        if let Some(icc) = &raster.icc {
            // SAFETY: live encoder; libjxl copies the profile.
            let status = unsafe { sys::JxlEncoderSetICCProfile(self.ptr, icc.as_ptr(), icc.len()) };
            if status == sys::JXL_ENC_SUCCESS {
                return Ok(());
            }
            // A malformed profile should not make the whole conversion fail:
            // fall back to sRGB, which is what browsers assume anyway.
            if quality == JxlQuality::Lossless {
                return self.check(status, "setting the colour profile");
            }
        }
        // SAFETY: SetTo* fully initialises the struct.
        let color = unsafe {
            let mut c = MaybeUninit::uninit();
            let gray = i32::from(raster.is_gray());
            if matches!(raster.samples, Samples::F32(_)) {
                sys::JxlColorEncodingSetToLinearSRGB(c.as_mut_ptr(), gray);
            } else {
                sys::JxlColorEncodingSetToSRGB(c.as_mut_ptr(), gray);
            }
            c.assume_init()
        };
        // SAFETY: live encoder, valid struct.
        unsafe {
            self.check(
                sys::JxlEncoderSetColorEncoding(self.ptr, &color),
                "setting the colour space",
            )
        }
    }

    /// Adds `Exif` and `xml ` boxes. Call before the first frame.
    fn add_metadata(&mut self, metadata: &Metadata) -> Result<()> {
        if metadata.is_empty() {
            return Ok(());
        }
        // SAFETY: live encoder; libjxl copies box contents.
        unsafe {
            self.check(sys::JxlEncoderUseBoxes(self.ptr), "enabling metadata boxes")?;
            if let Some(exif) = &metadata.exif {
                // JPEG XL's Exif box starts with a 4-byte offset to the TIFF
                // header; ours starts right away, so the offset is 0.
                let mut content = vec![0u8; 4];
                content.extend_from_slice(exif);
                let t = box_type(b"Exif");
                self.check(
                    sys::JxlEncoderAddBox(
                        self.ptr,
                        t.as_ptr(),
                        content.as_ptr(),
                        content.len(),
                        sys::JXL_FALSE,
                    ),
                    "adding EXIF",
                )?;
            }
            if let Some(xmp) = &metadata.xmp {
                let t = box_type(b"xml ");
                self.check(
                    sys::JxlEncoderAddBox(
                        self.ptr,
                        t.as_ptr(),
                        xmp.as_ptr(),
                        xmp.len(),
                        sys::JXL_FALSE,
                    ),
                    "adding XMP",
                )?;
            }
            sys::JxlEncoderCloseBoxes(self.ptr);
        }
        Ok(())
    }

    /// Creates frame settings with all options applied. The returned pointer
    /// is owned by the encoder and freed with it.
    fn frame_settings(
        &self,
        opts: &JxlEncodeOptions,
        quality: JxlQuality,
    ) -> Result<*mut sys::JxlEncoderFrameSettings> {
        // SAFETY: live encoder; settings live as long as the encoder.
        unsafe {
            let s = sys::JxlEncoderFrameSettingsCreate(self.ptr, std::ptr::null());
            if s.is_null() {
                return Err(Error::Encode("could not create frame settings".into()));
            }
            let set = |option, value: i64, what| {
                self.check(
                    sys::JxlEncoderFrameSettingsSetOption(s, option, value),
                    what,
                )
            };
            set(
                sys::JXL_ENC_FRAME_SETTING_EFFORT,
                opts.effort.clamp(1, 10).into(),
                "setting effort",
            )?;
            set(
                sys::JXL_ENC_FRAME_SETTING_DECODING_SPEED,
                opts.decoding_speed.min(4).into(),
                "setting decoding speed",
            )?;
            match quality {
                JxlQuality::Lossless => {
                    self.check(
                        sys::JxlEncoderSetFrameLossless(s, sys::JXL_TRUE),
                        "enabling lossless mode",
                    )?;
                }
                JxlQuality::Distance { distance } => {
                    self.check(
                        sys::JxlEncoderSetFrameDistance(s, distance.clamp(0.05, 25.0)),
                        "setting quality",
                    )?;
                }
            }
            if opts.progressive {
                // The same switches as `cjxl --progressive`.
                if quality == JxlQuality::Lossless {
                    set(
                        sys::JXL_ENC_FRAME_SETTING_RESPONSIVE,
                        1,
                        "enabling progressive mode",
                    )?;
                } else {
                    set(
                        sys::JXL_ENC_FRAME_SETTING_QPROGRESSIVE_AC,
                        1,
                        "enabling progressive mode",
                    )?;
                    // libjxl 0.12 fails to encode when a progressive DC frame
                    // is combined with progressive AC *and* centre-first group
                    // order. The DC (1:8 preview) is stored before all AC
                    // groups anyway, so with a focus point we skip it.
                    if opts.focus.is_none() {
                        set(
                            sys::JXL_ENC_FRAME_SETTING_PROGRESSIVE_DC,
                            1,
                            "enabling progressive mode",
                        )?;
                    }
                }
            }
            if let Some(focus) = opts.focus {
                // Group order 1 = "centre first", spiralling out from the point.
                set(
                    sys::JXL_ENC_FRAME_SETTING_GROUP_ORDER,
                    1,
                    "setting the subject-first order",
                )?;
                set(
                    sys::JXL_ENC_FRAME_SETTING_GROUP_ORDER_CENTER_X,
                    focus.x.into(),
                    "setting the subject-first order",
                )?;
                set(
                    sys::JXL_ENC_FRAME_SETTING_GROUP_ORDER_CENTER_Y,
                    focus.y.into(),
                    "setting the subject-first order",
                )?;
            }
            Ok(s)
        }
    }

    /// Adds one frame. `placement` = `(x, y, duration)`: when given, the frame
    /// is positioned at (x, y) and replaces what is below it.
    fn add_frame(
        &mut self,
        settings: *mut sys::JxlEncoderFrameSettings,
        raster: &Raster,
        placement: Option<(i32, i32, u32)>,
    ) -> Result<()> {
        // SAFETY: settings come from `frame_settings` on this encoder; pixel
        // buffers outlive the call (libjxl copies them).
        unsafe {
            if let Some((x, y, duration)) = placement {
                let mut header = {
                    let mut h = MaybeUninit::uninit();
                    sys::JxlEncoderInitFrameHeader(h.as_mut_ptr());
                    h.assume_init()
                };
                header.duration = duration;
                header.layer_info.have_crop = sys::JXL_TRUE;
                header.layer_info.crop_x0 = x;
                header.layer_info.crop_y0 = y;
                header.layer_info.xsize = raster.width;
                header.layer_info.ysize = raster.height;
                header.layer_info.blend_info.blendmode = sys::JXL_BLEND_REPLACE;
                self.check(
                    sys::JxlEncoderSetFrameHeader(settings, &header),
                    "positioning a layer",
                )?;
                if raster.has_alpha() {
                    let blend = header.layer_info.blend_info;
                    self.check(
                        sys::JxlEncoderSetExtraChannelBlendInfo(settings, 0, &blend),
                        "positioning a layer",
                    )?;
                }
            }
            let format = sys::JxlPixelFormat {
                num_channels: raster.channels.into(),
                data_type: match raster.samples {
                    Samples::U8(_) => sys::JXL_TYPE_UINT8,
                    Samples::U16(_) => sys::JXL_TYPE_UINT16,
                    Samples::F32(_) => sys::JXL_TYPE_FLOAT,
                },
                endianness: sys::JXL_NATIVE_ENDIAN,
                align: 0,
            };
            let (ptr, bytes) = match &raster.samples {
                Samples::U8(v) => (v.as_ptr().cast(), v.len()),
                Samples::U16(v) => (v.as_ptr().cast(), v.len() * 2),
                Samples::F32(v) => (v.as_ptr().cast(), v.len() * 4),
            };
            self.check(
                sys::JxlEncoderAddImageFrame(settings, &format, ptr, bytes),
                "adding pixels",
            )
        }
    }

    /// Closes the input and collects the encoded bytes.
    fn finish(self) -> Result<Vec<u8>> {
        // SAFETY: live encoder; `next_out`/`avail_out` always describe the
        // unused tail of `out`.
        unsafe {
            sys::JxlEncoderCloseInput(self.ptr);
            let mut out = vec![0u8; 64 * 1024];
            let mut written = 0usize;
            loop {
                let mut next_out = out.as_mut_ptr().add(written);
                let mut avail_out = out.len() - written;
                let status = sys::JxlEncoderProcessOutput(self.ptr, &mut next_out, &mut avail_out);
                written = out.len() - avail_out;
                match status {
                    sys::JXL_ENC_SUCCESS => break,
                    sys::JXL_ENC_NEED_MORE_OUTPUT => out.resize(out.len() * 2, 0),
                    other => return Err(self.check(other, "compressing").unwrap_err()),
                }
            }
            out.truncate(written);
            Ok(out)
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // SAFETY: created by JxlEncoderCreate, destroyed exactly once.
        unsafe { sys::JxlEncoderDestroy(self.ptr) }
    }
}

/// `(bits_per_sample, exponent_bits)` to declare for a raster.
fn sample_bits(raster: &Raster) -> (u32, u32) {
    match raster.samples {
        Samples::U8(_) => (8, 0),
        Samples::U16(_) => (raster.source_bits.clamp(9, 16), 0),
        Samples::F32(_) => (32, 8),
    }
}
