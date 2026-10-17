//! JPEG XL decoding: pixels, metadata, JPEG reconstruction, partial
//! (progressive) decoding and cheap probing.

use std::ffi::c_int;
use std::mem::MaybeUninit;

use jixel_libjxl_sys as sys;
use serde::Serialize;

use super::Runner;
use crate::metadata::Metadata;
use crate::{Error, Raster, Result, Samples};

/// A decoded JPEG XL file.
#[derive(Clone, Debug)]
pub struct JxlImage {
    /// Composited frames. A still image (even a layered one) has exactly one.
    pub frames: Vec<JxlFrame>,
    /// Animation repeat count (0 = forever). Meaningless for stills.
    pub loops: u32,
    pub probe: JxlProbe,
}

impl JxlImage {
    /// The first (for stills: the only) frame.
    pub fn into_first(self) -> Raster {
        self.frames.into_iter().next().map(|f| f.raster).expect("decoder returns at least one frame")
    }
}

#[derive(Clone, Debug)]
pub struct JxlFrame {
    pub raster: Raster,
    pub duration_ms: u32,
}

/// Everything about a JPEG XL file that can be learned without decoding
/// pixels. Shown in the app's *Inspect* screen.
#[derive(Clone, Debug, Default, Serialize)]
pub struct JxlProbe {
    pub width: u32,
    pub height: u32,
    pub bits_per_sample: u32,
    /// Non-zero for floating point samples (HDR, linear light).
    pub exponent_bits_per_sample: u32,
    pub color_channels: u32,
    pub has_alpha: bool,
    pub extra_channels: u32,
    /// True when stored in the original colour space, which JPEG XL does for
    /// lossless images (and JPEG recompression). Lossy images use XYB.
    pub uses_original_profile: bool,
    pub orientation: u32,
    pub has_preview: bool,
    pub is_animated: bool,
    /// Number of frames as stored (layers of a still image count here).
    pub frame_count: u32,
    /// Brightest intended display level, in nits. Above 255 hints at HDR.
    pub intensity_target: f32,
    /// e.g. "sRGB", "Linear", "PQ (HDR)", "HLG (HDR)", "ICC profile".
    pub color_description: String,
    pub has_icc: bool,
    /// The file stores what is needed to restore the original JPEG exactly.
    pub jpeg_reconstruction: bool,
    /// Uses the ISOBMFF container (needed for metadata boxes).
    pub has_container: bool,
    /// Container boxes, e.g. `["JXL ", "ftyp", "jbrd", "Exif", "jxlc"]`.
    pub boxes: Vec<String>,
}

impl JxlProbe {
    /// HDR content: float samples, PQ/HLG transfer, or a high intensity target.
    pub fn is_hdr(&self) -> bool {
        self.color_description.contains("HDR") || self.intensity_target > 255.0
    }
}

/// Decodes a complete JPEG XL file.
pub fn decode(bytes: &[u8]) -> Result<JxlImage> {
    let mut dec = Decoder::new(true)?;
    dec.subscribe(
        sys::JXL_DEC_BASIC_INFO | sys::JXL_DEC_COLOR_ENCODING | sys::JXL_DEC_FRAME | sys::JXL_DEC_FULL_IMAGE | sys::JXL_DEC_BOX,
    )?;
    dec.set_input(bytes, true)?;
    let mut frames = Vec::new();
    let mut duration = 0;
    loop {
        match dec.process() {
            sys::JXL_DEC_BASIC_INFO => dec.read_basic_info()?,
            sys::JXL_DEC_COLOR_ENCODING => dec.choose_output_color()?,
            sys::JXL_DEC_FRAME => duration = dec.frame_duration_ms()?,
            sys::JXL_DEC_NEED_IMAGE_OUT_BUFFER => dec.allocate_pixels()?,
            sys::JXL_DEC_FULL_IMAGE => frames.push(JxlFrame { raster: dec.take_raster()?, duration_ms: duration }),
            sys::JXL_DEC_BOX => dec.on_box()?,
            sys::JXL_DEC_BOX_NEED_MORE_OUTPUT => dec.grow_box()?,
            sys::JXL_DEC_SUCCESS => break,
            sys::JXL_DEC_NEED_MORE_INPUT => return Err(Error::Decode("the file is truncated".into())),
            _ => return Err(Error::Decode("the file is not a valid JPEG XL image".into())),
        }
    }
    dec.finish_box();
    if frames.is_empty() {
        return Err(Error::Decode("the file contains no image".into()));
    }
    let metadata = dec.metadata();
    for f in &mut frames {
        f.raster.metadata = metadata.clone();
    }
    Ok(JxlImage { frames, loops: dec.loops, probe: std::mem::take(&mut dec.probe) })
}

/// Decodes as much as possible from the first bytes of a file, the way a
/// browser shows an image that is still downloading. Returns `None` when
/// not even a first preview is available yet.
pub fn decode_prefix(prefix: &[u8]) -> Result<Option<Raster>> {
    let mut dec = Decoder::new(false)?;
    dec.subscribe(sys::JXL_DEC_BASIC_INFO | sys::JXL_DEC_COLOR_ENCODING | sys::JXL_DEC_FULL_IMAGE)?;
    // SAFETY: live decoder. Ask for every refinement step, not just DC.
    unsafe { sys::JxlDecoderSetProgressiveDetail(dec.ptr, sys::kPasses) };
    dec.set_input(prefix, false)?;
    loop {
        match dec.process() {
            sys::JXL_DEC_BASIC_INFO => dec.read_basic_info()?,
            sys::JXL_DEC_COLOR_ENCODING => dec.choose_output_color()?,
            sys::JXL_DEC_NEED_IMAGE_OUT_BUFFER => dec.allocate_pixels()?,
            sys::JXL_DEC_FULL_IMAGE => return dec.take_raster().map(Some),
            sys::JXL_DEC_NEED_MORE_INPUT => {
                if dec.pixels.is_none() {
                    return Ok(None);
                }
                // SAFETY: live decoder with an output buffer set.
                let flushed = unsafe { sys::JxlDecoderFlushImage(dec.ptr) };
                return if flushed == sys::JXL_DEC_SUCCESS { dec.take_raster().map(Some) } else { Ok(None) };
            }
            sys::JXL_DEC_SUCCESS => return Ok(None),
            _ => return Err(Error::Decode("the file is not a valid JPEG XL image".into())),
        }
    }
}

/// Restores the original JPEG from a losslessly recompressed JPEG XL file,
/// bit for bit.
pub fn reconstruct_jpeg(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut dec = Decoder::new(false)?;
    dec.subscribe(sys::JXL_DEC_JPEG_RECONSTRUCTION | sys::JXL_DEC_FULL_IMAGE)?;
    dec.set_input(bytes, true)?;
    let mut jpeg: Vec<u8> = Vec::new();
    let mut started = false;
    loop {
        // SAFETY: live decoder; the JPEG buffer is `jpeg[used..]` and stays
        // alive (and unmoved) until released.
        unsafe {
            match dec.process() {
                sys::JXL_DEC_JPEG_RECONSTRUCTION => {
                    started = true;
                    jpeg.resize(256 * 1024, 0);
                    sys::JxlDecoderSetJPEGBuffer(dec.ptr, jpeg.as_mut_ptr(), jpeg.len());
                }
                sys::JXL_DEC_JPEG_NEED_MORE_OUTPUT => {
                    let used = jpeg.len() - sys::JxlDecoderReleaseJPEGBuffer(dec.ptr);
                    jpeg.resize(jpeg.len() * 2, 0);
                    sys::JxlDecoderSetJPEGBuffer(dec.ptr, jpeg.as_mut_ptr().add(used), jpeg.len() - used);
                }
                sys::JXL_DEC_FULL_IMAGE if started => {
                    let used = jpeg.len() - sys::JxlDecoderReleaseJPEGBuffer(dec.ptr);
                    jpeg.truncate(used);
                    return Ok(jpeg);
                }
                // Pixels were requested: there is no JPEG to restore.
                sys::JXL_DEC_FULL_IMAGE | sys::JXL_DEC_NEED_IMAGE_OUT_BUFFER | sys::JXL_DEC_SUCCESS => {
                    return Err(Error::NoJpegReconstruction)
                }
                _ => return Err(Error::Decode("the file is not a valid JPEG XL image".into())),
            }
        }
    }
}

/// Reads image properties and container boxes without decoding pixels.
pub fn probe(bytes: &[u8]) -> Result<JxlProbe> {
    let mut dec = Decoder::new(false)?;
    dec.subscribe(sys::JXL_DEC_BASIC_INFO | sys::JXL_DEC_COLOR_ENCODING | sys::JXL_DEC_FRAME | sys::JXL_DEC_BOX)?;
    // Count stored frames (layers), not composited ones.
    // SAFETY: live decoder.
    unsafe { sys::JxlDecoderSetCoalescing(dec.ptr, sys::JXL_FALSE) };
    dec.set_input(bytes, true)?;
    loop {
        match dec.process() {
            sys::JXL_DEC_BASIC_INFO => dec.read_basic_info()?,
            sys::JXL_DEC_COLOR_ENCODING => dec.describe_color(),
            sys::JXL_DEC_FRAME => dec.probe.frame_count += 1,
            sys::JXL_DEC_BOX => dec.on_box()?,
            sys::JXL_DEC_BOX_NEED_MORE_OUTPUT => dec.grow_box()?,
            sys::JXL_DEC_SUCCESS => break,
            sys::JXL_DEC_NEED_MORE_INPUT => return Err(Error::Decode("the file is truncated".into())),
            _ => return Err(Error::Decode("the file is not a valid JPEG XL image".into())),
        }
    }
    Ok(std::mem::take(&mut dec.probe))
}

/// Pixel buffer of the sample type chosen for this image.
enum Pixels {
    U8(Vec<u8>),
    U16(Vec<u16>),
    F32(Vec<f32>),
}

/// Owns a `JxlDecoder` plus everything learned while decoding.
struct Decoder {
    ptr: *mut sys::JxlDecoder,
    _runner: Runner,
    info: Option<sys::JxlBasicInfo>,
    probe: JxlProbe,
    loops: u32,
    icc: Option<Vec<u8>>,
    pixels: Option<Pixels>,
    /// Box being collected: (type, buffer). Only `Exif` and `xml ` are kept.
    current_box: Option<([u8; 4], Vec<u8>)>,
    exif: Option<Vec<u8>>,
    xmp: Option<Vec<u8>>,
}

impl Decoder {
    fn new(threads: bool) -> Result<Self> {
        // SAFETY: null memory manager = default allocator.
        let ptr = unsafe { sys::JxlDecoderCreate(std::ptr::null()) };
        if ptr.is_null() {
            return Err(Error::Decode("out of memory".into()));
        }
        let runner = Runner::new();
        if threads && !runner.as_ptr().is_null() {
            // SAFETY: live decoder and runner; the runner outlives the decoder.
            unsafe { sys::JxlDecoderSetParallelRunner(ptr, sys::JxlThreadParallelRunner, runner.as_ptr()) };
        }
        // SAFETY: live decoder. Return compressed (`brob`) boxes decompressed.
        unsafe { sys::JxlDecoderSetDecompressBoxes(ptr, sys::JXL_TRUE) };
        Ok(Decoder {
            ptr,
            _runner: runner,
            info: None,
            probe: JxlProbe::default(),
            loops: 0,
            icc: None,
            pixels: None,
            current_box: None,
            exif: None,
            xmp: None,
        })
    }

    fn subscribe(&mut self, events: c_int) -> Result<()> {
        // SAFETY: live decoder.
        let status = unsafe { sys::JxlDecoderSubscribeEvents(self.ptr, events) };
        ok(status, "configuring the decoder")
    }

    /// `bytes` must stay alive as long as the decoder: guaranteed because
    /// every public function keeps both on its own stack frame.
    fn set_input(&mut self, bytes: &[u8], complete: bool) -> Result<()> {
        // SAFETY: live decoder; see the lifetime note above.
        unsafe {
            ok(sys::JxlDecoderSetInput(self.ptr, bytes.as_ptr(), bytes.len()), "reading the file")?;
            if complete {
                sys::JxlDecoderCloseInput(self.ptr);
            }
        }
        Ok(())
    }

    fn process(&mut self) -> sys::JxlDecoderStatus {
        // SAFETY: live decoder.
        unsafe { sys::JxlDecoderProcessInput(self.ptr) }
    }

    fn read_basic_info(&mut self) -> Result<()> {
        // SAFETY: called on the BASIC_INFO event, when the info is available.
        let info = unsafe {
            let mut info = MaybeUninit::uninit();
            ok(sys::JxlDecoderGetBasicInfo(self.ptr, info.as_mut_ptr()), "reading image properties")?;
            info.assume_init()
        };
        let p = &mut self.probe;
        p.width = info.xsize;
        p.height = info.ysize;
        p.bits_per_sample = info.bits_per_sample;
        p.exponent_bits_per_sample = info.exponent_bits_per_sample;
        p.color_channels = info.num_color_channels;
        p.has_alpha = info.alpha_bits > 0;
        p.extra_channels = info.num_extra_channels;
        p.uses_original_profile = info.uses_original_profile != 0;
        p.orientation = info.orientation as u32;
        p.has_preview = info.have_preview != 0;
        p.is_animated = info.have_animation != 0;
        p.intensity_target = info.intensity_target;
        p.has_container = info.have_container != 0;
        self.loops = info.animation.num_loops;
        self.info = Some(info);
        Ok(())
    }

    fn describe_color(&mut self) {
        // SAFETY: called on the COLOR_ENCODING event.
        unsafe {
            let mut enc = MaybeUninit::<sys::JxlColorEncoding>::uninit();
            let status =
                sys::JxlDecoderGetColorAsEncodedProfile(self.ptr, sys::JXL_COLOR_PROFILE_TARGET_ORIGINAL, enc.as_mut_ptr());
            self.probe.color_description = if status == sys::JXL_DEC_SUCCESS {
                match enc.assume_init().transfer_function {
                    sys::JXL_TRANSFER_FUNCTION_SRGB => "sRGB".into(),
                    sys::JXL_TRANSFER_FUNCTION_LINEAR => "Linear".into(),
                    sys::JXL_TRANSFER_FUNCTION_PQ => "PQ (HDR)".into(),
                    sys::JXL_TRANSFER_FUNCTION_HLG => "HLG (HDR)".into(),
                    _ => "Custom".into(),
                }
            } else {
                self.probe.has_icc = true;
                "ICC profile".into()
            };
        }
    }

    /// Picks the colour space of the output pixels: the image's *original*
    /// one, so that what we write next (PNG, JPEG…) looks the same. Then
    /// records the ICC profile that describes the output pixels.
    fn choose_output_color(&mut self) -> Result<()> {
        self.describe_color();
        let info = self.info.ok_or_else(|| Error::Decode("missing image properties".into()))?;
        // SAFETY: called on the COLOR_ENCODING event, before buffers are set.
        unsafe {
            if info.uses_original_profile == 0 {
                // Lossy (XYB) images can be decoded to any colour space.
                let mut enc = MaybeUninit::<sys::JxlColorEncoding>::uninit();
                let described = sys::JxlDecoderGetColorAsEncodedProfile(
                    self.ptr,
                    sys::JXL_COLOR_PROFILE_TARGET_ORIGINAL,
                    enc.as_mut_ptr(),
                ) == sys::JXL_DEC_SUCCESS;
                if described {
                    sys::JxlDecoderSetPreferredColorProfile(self.ptr, enc.as_ptr());
                } else if let Some(icc) = self.read_icc(sys::JXL_COLOR_PROFILE_TARGET_ORIGINAL) {
                    // Arbitrary ICC profile: needs a colour management system.
                    sys::JxlDecoderSetCms(self.ptr, *sys::JxlGetDefaultCms());
                    sys::JxlDecoderSetOutputColorProfile(self.ptr, std::ptr::null(), icc.as_ptr(), icc.len());
                }
            }
            self.icc = self.read_icc(sys::JXL_COLOR_PROFILE_TARGET_DATA);
        }
        Ok(())
    }

    fn read_icc(&self, target: sys::JxlColorProfileTarget) -> Option<Vec<u8>> {
        // SAFETY: called after the COLOR_ENCODING event.
        unsafe {
            let mut size = 0usize;
            if sys::JxlDecoderGetICCProfileSize(self.ptr, target, &mut size) != sys::JXL_DEC_SUCCESS || size == 0 {
                return None;
            }
            let mut icc = vec![0u8; size];
            (sys::JxlDecoderGetColorAsICCProfile(self.ptr, target, icc.as_mut_ptr(), size) == sys::JXL_DEC_SUCCESS)
                .then_some(icc)
        }
    }

    fn frame_duration_ms(&self) -> Result<u32> {
        let info = self.info.ok_or_else(|| Error::Decode("missing image properties".into()))?;
        // SAFETY: called on the FRAME event.
        let header = unsafe {
            let mut h = MaybeUninit::uninit();
            ok(sys::JxlDecoderGetFrameHeader(self.ptr, h.as_mut_ptr()), "reading a frame")?;
            h.assume_init()
        };
        let a = info.animation;
        if a.tps_numerator == 0 {
            return Ok(0);
        }
        Ok((header.duration as u64 * 1000 * a.tps_denominator as u64 / a.tps_numerator as u64) as u32)
    }

    fn pixel_format(&self) -> Result<sys::JxlPixelFormat> {
        let info = self.info.ok_or_else(|| Error::Decode("missing image properties".into()))?;
        let channels = info.num_color_channels + u32::from(info.alpha_bits > 0);
        let data_type = if info.exponent_bits_per_sample > 0 {
            sys::JXL_TYPE_FLOAT
        } else if info.bits_per_sample <= 8 {
            sys::JXL_TYPE_UINT8
        } else {
            sys::JXL_TYPE_UINT16
        };
        Ok(sys::JxlPixelFormat { num_channels: channels, data_type, endianness: sys::JXL_NATIVE_ENDIAN, align: 0 })
    }

    fn allocate_pixels(&mut self) -> Result<()> {
        let format = self.pixel_format()?;
        // SAFETY: called on NEED_IMAGE_OUT_BUFFER. The buffer is stored in
        // `self.pixels` and is not moved or freed until the frame is taken.
        unsafe {
            let mut size = 0usize;
            ok(sys::JxlDecoderImageOutBufferSize(self.ptr, &format, &mut size), "sizing the output")?;
            let (pixels, ptr) = match format.data_type {
                sys::JXL_TYPE_UINT8 => {
                    let mut v = vec![0u8; size];
                    let p = v.as_mut_ptr().cast();
                    (Pixels::U8(v), p)
                }
                sys::JXL_TYPE_UINT16 => {
                    let mut v = vec![0u16; size / 2];
                    let p = v.as_mut_ptr().cast();
                    (Pixels::U16(v), p)
                }
                _ => {
                    let mut v = vec![0f32; size / 4];
                    let p = v.as_mut_ptr().cast();
                    (Pixels::F32(v), p)
                }
            };
            // Moving a Vec into the enum does not move its heap buffer.
            self.pixels = Some(pixels);
            ok(sys::JxlDecoderSetImageOutBuffer(self.ptr, &format, ptr, size), "setting the output")
        }
    }

    fn take_raster(&mut self) -> Result<Raster> {
        let info = self.info.ok_or_else(|| Error::Decode("missing image properties".into()))?;
        let format = self.pixel_format()?;
        let samples = match &self.pixels {
            Some(Pixels::U8(v)) => Samples::U8(v.clone()),
            Some(Pixels::U16(v)) => Samples::U16(v.clone()),
            Some(Pixels::F32(v)) => Samples::F32(v.clone()),
            None => return Err(Error::Decode("no pixels were decoded".into())),
        };
        // libjxl applies the orientation by default, so the output may be
        // transposed relative to the stored size.
        let (width, height) = if info.orientation > 4 { (info.ysize, info.xsize) } else { (info.xsize, info.ysize) };
        Ok(Raster {
            width,
            height,
            channels: format.num_channels as u8,
            samples,
            source_bits: info.bits_per_sample,
            icc: self.icc.clone(),
            orientation: 1,
            // Filled in by `decode` once all boxes have been read.
            metadata: Metadata::default(),
        })
    }

    // ---- metadata boxes

    fn on_box(&mut self) -> Result<()> {
        self.finish_box();
        let mut t = [0 as std::ffi::c_char; 4];
        // SAFETY: called on the BOX event.
        unsafe { ok(sys::JxlDecoderGetBoxType(self.ptr, t.as_mut_ptr(), sys::JXL_TRUE), "reading a box")? };
        let t = t.map(|c| c as u8);
        self.probe.boxes.push(String::from_utf8_lossy(&t).into_owned());
        if &t == b"jbrd" {
            self.probe.jpeg_reconstruction = true;
        }
        if &t == b"Exif" || &t == b"xml " {
            let mut buf = vec![0u8; 16 * 1024];
            // SAFETY: the buffer is kept in `current_box` until released.
            unsafe { sys::JxlDecoderSetBoxBuffer(self.ptr, buf.as_mut_ptr(), buf.len()) };
            self.current_box = Some((t, buf));
        }
        Ok(())
    }

    fn grow_box(&mut self) -> Result<()> {
        let Some((_, buf)) = self.current_box.as_mut() else {
            return Err(Error::Decode("unexpected box data".into()));
        };
        // SAFETY: re-points libjxl at the unused tail of the (grown) buffer.
        unsafe {
            let used = buf.len() - sys::JxlDecoderReleaseBoxBuffer(self.ptr);
            buf.resize(buf.len() * 2, 0);
            sys::JxlDecoderSetBoxBuffer(self.ptr, buf.as_mut_ptr().add(used), buf.len() - used);
        }
        Ok(())
    }

    fn finish_box(&mut self) {
        let Some((t, mut buf)) = self.current_box.take() else { return };
        // SAFETY: releases the buffer set in `on_box` / `grow_box`.
        let unused = unsafe { sys::JxlDecoderReleaseBoxBuffer(self.ptr) };
        buf.truncate(buf.len() - unused);
        match &t {
            b"Exif" => self.exif = Metadata::from_exif_any(&buf),
            _ => self.xmp = Some(buf),
        }
    }

    fn metadata(&self) -> Metadata {
        let mut m = Metadata { exif: self.exif.clone(), xmp: self.xmp.clone() };
        // Pixels come out already rotated: keep EXIF consistent with them.
        m.reset_exif_orientation();
        m
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: created by JxlDecoderCreate, destroyed exactly once, before
        // the buffers it may point into (fields drop after this).
        unsafe { sys::JxlDecoderDestroy(self.ptr) }
    }
}

fn ok(status: sys::JxlDecoderStatus, context: &'static str) -> Result<()> {
    if status == sys::JXL_DEC_SUCCESS {
        Ok(())
    } else {
        Err(Error::Libjxl { context, code: status })
    }
}
