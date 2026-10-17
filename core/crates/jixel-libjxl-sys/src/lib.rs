//! Minimal, hand-written FFI bindings to [libjxl](https://github.com/libjxl/libjxl).
//!
//! # Why hand-written?
//!
//! The popular `jpegxl-sys` crate is GPL-3.0. Jixel is Apache-2.0 and must be
//! distributable through app stores, so we declare only the part of the C API
//! we use, directly from libjxl's BSD-licensed headers
//! (`libjxl/lib/include/jxl/*.h`). See `docs/adr/0002-own-libjxl-bindings.md`.
//!
//! # Conventions
//!
//! * C enums are represented as plain `c_int` type aliases with `const`
//!   values. A Rust `enum` would be undefined behaviour if libjxl ever returned
//!   a value we did not list; an integer cannot be invalid.
//! * Structs are `#[repr(C)]` and mirror the C field order exactly. Each one
//!   names the header it comes from: when upgrading libjxl, diff that header.
//! * Nothing here is safe to call directly. The safe API lives in
//!   `jixel-core::codec::jxl`.

#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    clippy::upper_case_acronyms
)]

use std::ffi::{c_char, c_int, c_void};

/// `JXL_BOOL` (types.h): an `int` that is either 0 or 1.
pub type JxlBool = c_int;
pub const JXL_TRUE: JxlBool = 1;
pub const JXL_FALSE: JxlBool = 0;

// ---------------------------------------------------------------- types.h

pub type JxlDataType = c_int;
pub const JXL_TYPE_FLOAT: JxlDataType = 0;
pub const JXL_TYPE_UINT8: JxlDataType = 2;
pub const JXL_TYPE_UINT16: JxlDataType = 3;
pub const JXL_TYPE_FLOAT16: JxlDataType = 5;

pub type JxlEndianness = c_int;
pub const JXL_NATIVE_ENDIAN: JxlEndianness = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlPixelFormat {
    pub num_channels: u32,
    pub data_type: JxlDataType,
    pub endianness: JxlEndianness,
    pub align: usize,
}

pub type JxlBitDepthType = c_int;
pub const JXL_BIT_DEPTH_FROM_PIXEL_FORMAT: JxlBitDepthType = 0;
pub const JXL_BIT_DEPTH_FROM_CODESTREAM: JxlBitDepthType = 1;
pub const JXL_BIT_DEPTH_CUSTOM: JxlBitDepthType = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlBitDepth {
    pub type_: JxlBitDepthType,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
}

/// Four ASCII characters identifying an ISOBMFF box, e.g. `b"Exif"`.
pub type JxlBoxType = [c_char; 4];

// ---------------------------------------------------- codestream_header.h

pub type JxlOrientation = c_int;
pub type JxlExtraChannelType = c_int;
pub const JXL_CHANNEL_ALPHA: JxlExtraChannelType = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlPreviewHeader {
    pub xsize: u32,
    pub ysize: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlAnimationHeader {
    pub tps_numerator: u32,
    pub tps_denominator: u32,
    pub num_loops: u32,
    pub have_timecodes: JxlBool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlBasicInfo {
    pub have_container: JxlBool,
    pub xsize: u32,
    pub ysize: u32,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub intensity_target: f32,
    pub min_nits: f32,
    pub relative_to_max_display: JxlBool,
    pub linear_below: f32,
    pub uses_original_profile: JxlBool,
    pub have_preview: JxlBool,
    pub have_animation: JxlBool,
    pub orientation: JxlOrientation,
    pub num_color_channels: u32,
    pub num_extra_channels: u32,
    pub alpha_bits: u32,
    pub alpha_exponent_bits: u32,
    pub alpha_premultiplied: JxlBool,
    pub preview: JxlPreviewHeader,
    pub animation: JxlAnimationHeader,
    pub intrinsic_xsize: u32,
    pub intrinsic_ysize: u32,
    pub padding: [u8; 100],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlExtraChannelInfo {
    pub type_: JxlExtraChannelType,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub dim_shift: u32,
    pub name_length: u32,
    pub alpha_premultiplied: JxlBool,
    pub spot_color: [f32; 4],
    pub cfa_channel: u32,
}

pub type JxlBlendMode = c_int;
pub const JXL_BLEND_REPLACE: JxlBlendMode = 0;
pub const JXL_BLEND_ADD: JxlBlendMode = 1;
pub const JXL_BLEND_BLEND: JxlBlendMode = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlBlendInfo {
    pub blendmode: JxlBlendMode,
    pub source: u32,
    pub alpha: u32,
    pub clamp: JxlBool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlLayerInfo {
    pub have_crop: JxlBool,
    pub crop_x0: i32,
    pub crop_y0: i32,
    pub xsize: u32,
    pub ysize: u32,
    pub blend_info: JxlBlendInfo,
    pub save_as_reference: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlFrameHeader {
    pub duration: u32,
    pub timecode: u32,
    pub name_length: u32,
    pub is_last: JxlBool,
    pub layer_info: JxlLayerInfo,
}

// ------------------------------------------------------- color_encoding.h

pub type JxlColorSpace = c_int;
pub type JxlWhitePoint = c_int;
pub type JxlPrimaries = c_int;
pub type JxlTransferFunction = c_int;
pub const JXL_TRANSFER_FUNCTION_LINEAR: JxlTransferFunction = 8;
pub const JXL_TRANSFER_FUNCTION_SRGB: JxlTransferFunction = 13;
pub const JXL_TRANSFER_FUNCTION_PQ: JxlTransferFunction = 16;
pub const JXL_TRANSFER_FUNCTION_HLG: JxlTransferFunction = 18;
pub type JxlRenderingIntent = c_int;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlColorEncoding {
    pub color_space: JxlColorSpace,
    pub white_point: JxlWhitePoint,
    pub white_point_xy: [f64; 2],
    pub primaries: JxlPrimaries,
    pub primaries_red_xy: [f64; 2],
    pub primaries_green_xy: [f64; 2],
    pub primaries_blue_xy: [f64; 2],
    pub transfer_function: JxlTransferFunction,
    pub gamma: f64,
    pub rendering_intent: JxlRenderingIntent,
}

// ------------------------------------------------- parallel_runner.h & co.

/// `JxlParallelRunner` (parallel_runner.h). The two callback parameters are
/// function pointers we never call ourselves, so they are passed as opaque
/// pointers; this is ABI-identical on every supported platform.
pub type JxlParallelRunner = unsafe extern "C" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init: *const c_void,
    func: *const c_void,
    start_range: u32,
    end_range: u32,
) -> c_int;

/// Opaque memory manager (memory_manager.h). We always pass null (= malloc).
#[repr(C)]
pub struct JxlMemoryManager {
    _private: [u8; 0],
}

// -------------------------------------------------------- cms_interface.h

/// `JxlCmsInterface` (cms_interface.h): eight pointers. We only ever obtain it
/// from [`JxlGetDefaultCms`] and hand it back to libjxl, so the callback
/// fields are kept opaque.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JxlCmsInterface {
    pub set_fields_data: *mut c_void,
    pub set_fields_from_icc: *const c_void,
    pub init_data: *mut c_void,
    pub init: *const c_void,
    pub get_src_buf: *const c_void,
    pub get_dst_buf: *const c_void,
    pub run: *const c_void,
    pub destroy: *const c_void,
}

// --------------------------------------------------------------- decode.h

#[repr(C)]
pub struct JxlDecoder {
    _private: [u8; 0],
}

pub type JxlSignature = c_int;
pub const JXL_SIG_NOT_ENOUGH_BYTES: JxlSignature = 0;
pub const JXL_SIG_INVALID: JxlSignature = 1;
pub const JXL_SIG_CODESTREAM: JxlSignature = 2;
pub const JXL_SIG_CONTAINER: JxlSignature = 3;

pub type JxlDecoderStatus = c_int;
pub const JXL_DEC_SUCCESS: JxlDecoderStatus = 0;
pub const JXL_DEC_ERROR: JxlDecoderStatus = 1;
pub const JXL_DEC_NEED_MORE_INPUT: JxlDecoderStatus = 2;
pub const JXL_DEC_NEED_PREVIEW_OUT_BUFFER: JxlDecoderStatus = 3;
pub const JXL_DEC_NEED_IMAGE_OUT_BUFFER: JxlDecoderStatus = 5;
pub const JXL_DEC_JPEG_NEED_MORE_OUTPUT: JxlDecoderStatus = 6;
pub const JXL_DEC_BOX_NEED_MORE_OUTPUT: JxlDecoderStatus = 7;
pub const JXL_DEC_BASIC_INFO: JxlDecoderStatus = 0x40;
pub const JXL_DEC_COLOR_ENCODING: JxlDecoderStatus = 0x100;
pub const JXL_DEC_PREVIEW_IMAGE: JxlDecoderStatus = 0x200;
pub const JXL_DEC_FRAME: JxlDecoderStatus = 0x400;
pub const JXL_DEC_FULL_IMAGE: JxlDecoderStatus = 0x1000;
pub const JXL_DEC_JPEG_RECONSTRUCTION: JxlDecoderStatus = 0x2000;
pub const JXL_DEC_BOX: JxlDecoderStatus = 0x4000;
pub const JXL_DEC_FRAME_PROGRESSION: JxlDecoderStatus = 0x8000;
pub const JXL_DEC_BOX_COMPLETE: JxlDecoderStatus = 0x10000;

pub type JxlProgressiveDetail = c_int;
pub const kFrames: JxlProgressiveDetail = 0;
pub const kDC: JxlProgressiveDetail = 1;
pub const kLastPasses: JxlProgressiveDetail = 2;
pub const kPasses: JxlProgressiveDetail = 3;
pub const kDCProgressive: JxlProgressiveDetail = 4;
pub const kDCGroups: JxlProgressiveDetail = 5;
pub const kGroups: JxlProgressiveDetail = 6;

pub type JxlColorProfileTarget = c_int;
pub const JXL_COLOR_PROFILE_TARGET_ORIGINAL: JxlColorProfileTarget = 0;
pub const JXL_COLOR_PROFILE_TARGET_DATA: JxlColorProfileTarget = 1;

// --------------------------------------------------------------- encode.h

#[repr(C)]
pub struct JxlEncoder {
    _private: [u8; 0],
}

#[repr(C)]
pub struct JxlEncoderFrameSettings {
    _private: [u8; 0],
}

pub type JxlEncoderStatus = c_int;
pub const JXL_ENC_SUCCESS: JxlEncoderStatus = 0;
pub const JXL_ENC_ERROR: JxlEncoderStatus = 1;
pub const JXL_ENC_NEED_MORE_OUTPUT: JxlEncoderStatus = 2;

pub type JxlEncoderError = c_int;
pub const JXL_ENC_ERR_OK: JxlEncoderError = 0;
pub const JXL_ENC_ERR_GENERIC: JxlEncoderError = 1;
pub const JXL_ENC_ERR_OOM: JxlEncoderError = 2;
pub const JXL_ENC_ERR_JBRD: JxlEncoderError = 3;
pub const JXL_ENC_ERR_BAD_INPUT: JxlEncoderError = 4;
pub const JXL_ENC_ERR_NOT_SUPPORTED: JxlEncoderError = 0x80;
pub const JXL_ENC_ERR_API_USAGE: JxlEncoderError = 0x81;

/// `JxlEncoderFrameSettingId` (encode.h). Only the options Jixel uses.
pub type JxlEncoderFrameSettingId = c_int;
pub const JXL_ENC_FRAME_SETTING_EFFORT: JxlEncoderFrameSettingId = 0;
pub const JXL_ENC_FRAME_SETTING_DECODING_SPEED: JxlEncoderFrameSettingId = 1;
pub const JXL_ENC_FRAME_SETTING_RESAMPLING: JxlEncoderFrameSettingId = 2;
pub const JXL_ENC_FRAME_SETTING_PHOTON_NOISE: JxlEncoderFrameSettingId = 5;
pub const JXL_ENC_FRAME_SETTING_MODULAR: JxlEncoderFrameSettingId = 11;
pub const JXL_ENC_FRAME_SETTING_GROUP_ORDER: JxlEncoderFrameSettingId = 13;
pub const JXL_ENC_FRAME_SETTING_GROUP_ORDER_CENTER_X: JxlEncoderFrameSettingId = 14;
pub const JXL_ENC_FRAME_SETTING_GROUP_ORDER_CENTER_Y: JxlEncoderFrameSettingId = 15;
pub const JXL_ENC_FRAME_SETTING_RESPONSIVE: JxlEncoderFrameSettingId = 16;
pub const JXL_ENC_FRAME_SETTING_PROGRESSIVE_AC: JxlEncoderFrameSettingId = 17;
pub const JXL_ENC_FRAME_SETTING_QPROGRESSIVE_AC: JxlEncoderFrameSettingId = 18;
pub const JXL_ENC_FRAME_SETTING_PROGRESSIVE_DC: JxlEncoderFrameSettingId = 19;
pub const JXL_ENC_FRAME_SETTING_BROTLI_EFFORT: JxlEncoderFrameSettingId = 32;
pub const JXL_ENC_FRAME_SETTING_JPEG_COMPRESS_BOXES: JxlEncoderFrameSettingId = 33;
pub const JXL_ENC_FRAME_SETTING_JPEG_KEEP_EXIF: JxlEncoderFrameSettingId = 35;
pub const JXL_ENC_FRAME_SETTING_JPEG_KEEP_XMP: JxlEncoderFrameSettingId = 36;
pub const JXL_ENC_FRAME_SETTING_JPEG_KEEP_JUMBF: JxlEncoderFrameSettingId = 37;

extern "C" {
    // ---- decode.h
    pub fn JxlDecoderVersion() -> u32;
    pub fn JxlSignatureCheck(buf: *const u8, len: usize) -> JxlSignature;
    pub fn JxlDecoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlDecoder;
    pub fn JxlDecoderDestroy(dec: *mut JxlDecoder);
    pub fn JxlDecoderSetParallelRunner(
        dec: *mut JxlDecoder,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSubscribeEvents(
        dec: *mut JxlDecoder,
        events_wanted: c_int,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetKeepOrientation(
        dec: *mut JxlDecoder,
        skip_reorientation: JxlBool,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetUnpremultiplyAlpha(
        dec: *mut JxlDecoder,
        unpremul_alpha: JxlBool,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetCoalescing(dec: *mut JxlDecoder, coalescing: JxlBool) -> JxlDecoderStatus;
    pub fn JxlDecoderProcessInput(dec: *mut JxlDecoder) -> JxlDecoderStatus;
    pub fn JxlDecoderSetInput(
        dec: *mut JxlDecoder,
        data: *const u8,
        size: usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderReleaseInput(dec: *mut JxlDecoder) -> usize;
    pub fn JxlDecoderCloseInput(dec: *mut JxlDecoder);
    pub fn JxlDecoderGetBasicInfo(
        dec: *const JxlDecoder,
        info: *mut JxlBasicInfo,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetExtraChannelInfo(
        dec: *const JxlDecoder,
        index: usize,
        info: *mut JxlExtraChannelInfo,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetColorAsEncodedProfile(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        color_encoding: *mut JxlColorEncoding,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetICCProfileSize(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        size: *mut usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetColorAsICCProfile(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        icc_profile: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetPreferredColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetFrameHeader(
        dec: *const JxlDecoder,
        header: *mut JxlFrameHeader,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderImageOutBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetImageOutBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetImageOutBitDepth(
        dec: *mut JxlDecoder,
        bit_depth: *const JxlBitDepth,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetJPEGBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderReleaseJPEGBuffer(dec: *mut JxlDecoder) -> usize;
    pub fn JxlDecoderSetBoxBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderReleaseBoxBuffer(dec: *mut JxlDecoder) -> usize;
    pub fn JxlDecoderSetDecompressBoxes(
        dec: *mut JxlDecoder,
        decompress: JxlBool,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetBoxType(
        dec: *mut JxlDecoder,
        type_: *mut c_char,
        decompressed: JxlBool,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderGetBoxSizeRaw(dec: *const JxlDecoder, size: *mut u64) -> JxlDecoderStatus;
    pub fn JxlDecoderSetProgressiveDetail(
        dec: *mut JxlDecoder,
        detail: JxlProgressiveDetail,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderFlushImage(dec: *mut JxlDecoder) -> JxlDecoderStatus;
    pub fn JxlDecoderSetOutputColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
        icc_data: *const u8,
        icc_size: usize,
    ) -> JxlDecoderStatus;
    pub fn JxlDecoderSetCms(dec: *mut JxlDecoder, cms: JxlCmsInterface) -> JxlDecoderStatus;

    // ---- encode.h
    pub fn JxlEncoderVersion() -> u32;
    pub fn JxlEncoderSetCms(enc: *mut JxlEncoder, cms: JxlCmsInterface);
    pub fn JxlEncoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlEncoder;
    pub fn JxlEncoderDestroy(enc: *mut JxlEncoder);
    pub fn JxlEncoderSetParallelRunner(
        enc: *mut JxlEncoder,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderGetError(enc: *mut JxlEncoder) -> JxlEncoderError;
    pub fn JxlEncoderProcessOutput(
        enc: *mut JxlEncoder,
        next_out: *mut *mut u8,
        avail_out: *mut usize,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderSetFrameHeader(
        frame_settings: *mut JxlEncoderFrameSettings,
        frame_header: *const JxlFrameHeader,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderSetExtraChannelBlendInfo(
        frame_settings: *mut JxlEncoderFrameSettings,
        index: usize,
        blend_info: *const JxlBlendInfo,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderSetFrameBitDepth(
        frame_settings: *mut JxlEncoderFrameSettings,
        bit_depth: *const JxlBitDepth,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderAddJPEGFrame(
        frame_settings: *const JxlEncoderFrameSettings,
        buffer: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderAddImageFrame(
        frame_settings: *const JxlEncoderFrameSettings,
        pixel_format: *const JxlPixelFormat,
        buffer: *const c_void,
        size: usize,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderAddBox(
        enc: *mut JxlEncoder,
        type_: *const c_char,
        contents: *const u8,
        size: usize,
        compress_box: JxlBool,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderUseBoxes(enc: *mut JxlEncoder) -> JxlEncoderStatus;
    pub fn JxlEncoderCloseBoxes(enc: *mut JxlEncoder);
    pub fn JxlEncoderCloseFrames(enc: *mut JxlEncoder);
    pub fn JxlEncoderCloseInput(enc: *mut JxlEncoder);
    pub fn JxlEncoderSetColorEncoding(
        enc: *mut JxlEncoder,
        color: *const JxlColorEncoding,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderSetICCProfile(
        enc: *mut JxlEncoder,
        icc_profile: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderInitBasicInfo(info: *mut JxlBasicInfo);
    pub fn JxlEncoderInitFrameHeader(frame_header: *mut JxlFrameHeader);
    pub fn JxlEncoderInitBlendInfo(blend_info: *mut JxlBlendInfo);
    pub fn JxlEncoderSetBasicInfo(
        enc: *mut JxlEncoder,
        info: *const JxlBasicInfo,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderFrameSettingsSetOption(
        frame_settings: *mut JxlEncoderFrameSettings,
        option: JxlEncoderFrameSettingId,
        value: i64,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderUseContainer(enc: *mut JxlEncoder, use_container: JxlBool)
        -> JxlEncoderStatus;
    pub fn JxlEncoderStoreJPEGMetadata(
        enc: *mut JxlEncoder,
        store_jpeg_metadata: JxlBool,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderSetCodestreamLevel(enc: *mut JxlEncoder, level: c_int) -> JxlEncoderStatus;
    pub fn JxlEncoderSetFrameLossless(
        frame_settings: *mut JxlEncoderFrameSettings,
        lossless: JxlBool,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderSetFrameDistance(
        frame_settings: *mut JxlEncoderFrameSettings,
        distance: f32,
    ) -> JxlEncoderStatus;
    pub fn JxlEncoderDistanceFromQuality(quality: f32) -> f32;
    pub fn JxlEncoderFrameSettingsCreate(
        enc: *mut JxlEncoder,
        source: *const JxlEncoderFrameSettings,
    ) -> *mut JxlEncoderFrameSettings;
    pub fn JxlColorEncodingSetToSRGB(color_encoding: *mut JxlColorEncoding, is_gray: JxlBool);
    pub fn JxlColorEncodingSetToLinearSRGB(color_encoding: *mut JxlColorEncoding, is_gray: JxlBool);

    // ---- cms.h (lives in the separate `jxl_cms` library)
    pub fn JxlGetDefaultCms() -> *const JxlCmsInterface;

    // ---- thread_parallel_runner.h
    pub fn JxlThreadParallelRunner(
        runner_opaque: *mut c_void,
        jpegxl_opaque: *mut c_void,
        init: *const c_void,
        func: *const c_void,
        start_range: u32,
        end_range: u32,
    ) -> c_int;
    pub fn JxlThreadParallelRunnerCreate(
        memory_manager: *const JxlMemoryManager,
        num_worker_threads: usize,
    ) -> *mut c_void;
    pub fn JxlThreadParallelRunnerDestroy(runner_opaque: *mut c_void);
    pub fn JxlThreadParallelRunnerDefaultNumWorkerThreads() -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_against_libjxl_0_12() {
        // 0.12.0 is encoded as 12000 (major * 1_000_000 + minor * 1000 + patch).
        assert_eq!(unsafe { JxlDecoderVersion() }, 12000);
        assert_eq!(unsafe { JxlEncoderVersion() }, 12000);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn struct_sizes_match_libjxl_headers() {
        // Sizes measured with `sizeof` on the C headers (64-bit targets).
        assert_eq!(std::mem::size_of::<JxlPixelFormat>(), 24);
        assert_eq!(std::mem::size_of::<JxlBlendInfo>(), 16);
        assert_eq!(std::mem::size_of::<JxlLayerInfo>(), 40);
        assert_eq!(std::mem::size_of::<JxlFrameHeader>(), 56);
        assert_eq!(std::mem::size_of::<JxlBasicInfo>(), 204);
        assert_eq!(std::mem::size_of::<JxlExtraChannelInfo>(), 44);
        assert_eq!(std::mem::size_of::<JxlColorEncoding>(), 104);
        assert_eq!(std::mem::size_of::<JxlBitDepth>(), 12);
        assert_eq!(
            std::mem::size_of::<JxlCmsInterface>(),
            8 * std::mem::size_of::<usize>()
        );
    }
}
