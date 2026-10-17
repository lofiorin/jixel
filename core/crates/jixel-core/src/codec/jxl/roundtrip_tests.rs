//! End-to-end tests of the JPEG XL wrapper against real libjxl.

use super::*;
use crate::codec::raster::{self, RasterEncodeOptions};
use crate::metadata::{Metadata, MetadataPolicy};
use crate::test_support::photo;
use crate::{Format, Raster, Samples};

fn lossless() -> JxlEncodeOptions {
    JxlEncodeOptions {
        quality: JxlQuality::Lossless,
        effort: 3,
        ..Default::default()
    }
}

#[test]
fn lossless_round_trip_is_exact_for_rgb_rgba_and_16_bit() {
    let mut sixteen = photo(33, 21, 3);
    sixteen.samples = match &sixteen.samples {
        Samples::U8(v) => Samples::U16(v.iter().map(|&b| b as u16 * 257 + 3).collect()),
        _ => unreachable!(),
    };
    sixteen.source_bits = 16;
    for original in [photo(33, 21, 3), photo(64, 64, 4), sixteen] {
        let bytes = encode(&original, &lossless()).unwrap();
        assert_eq!(Format::detect(&bytes), Some(Format::Jxl));
        let back = decode(&bytes).unwrap().into_first();
        assert_eq!(
            back.samples, original.samples,
            "{} channels",
            original.channels
        );
        assert_eq!(
            (back.width, back.height, back.channels),
            (original.width, original.height, original.channels)
        );
    }
}

#[test]
fn lossy_is_smaller_than_lossless() {
    let img = photo(256, 256, 3);
    let lossy = encode(
        &img,
        &JxlEncodeOptions {
            effort: 3,
            ..Default::default()
        },
    )
    .unwrap();
    let exact = encode(&img, &lossless()).unwrap();
    assert!(
        lossy.len() < exact.len(),
        "{} vs {}",
        lossy.len(),
        exact.len()
    );
    let back = decode(&lossy).unwrap().into_first();
    assert_eq!((back.width, back.height), (256, 256));
}

#[test]
fn jpeg_recompression_is_bit_exact_and_smaller() {
    let jpeg = raster::encode(
        &photo(200, 150, 3),
        Format::Jpeg,
        &RasterEncodeOptions {
            quality: 90.0,
            ..Default::default()
        },
    )
    .unwrap();
    let opts = JxlEncodeOptions {
        metadata: MetadataPolicy::Keep,
        ..Default::default()
    };
    let jxl = transcode_jpeg(&jpeg, &opts).unwrap();
    assert!(
        jxl.len() < jpeg.len(),
        "jxl {} >= jpeg {}",
        jxl.len(),
        jpeg.len()
    );
    assert_eq!(
        reconstruct_jpeg(&jxl).unwrap(),
        jpeg,
        "restored JPEG must be identical"
    );
    let p = probe(&jxl).unwrap();
    assert!(p.jpeg_reconstruction);
    assert!(p.boxes.iter().any(|b| b == "jbrd"));
    // Pixels decode too.
    assert_eq!(decode(&jxl).unwrap().into_first().width, 200);
}

#[test]
fn reconstruct_fails_cleanly_for_non_jpeg_sources() {
    let jxl = encode(&photo(16, 16, 3), &lossless()).unwrap();
    assert!(matches!(
        reconstruct_jpeg(&jxl),
        Err(crate::Error::NoJpegReconstruction)
    ));
}

#[test]
fn layers_are_composited_into_one_image() {
    let base = Raster::from_u8(64, 64, 3, vec![10; 64 * 64 * 3]);
    let patch = Raster::from_u8(16, 8, 3, vec![200; 16 * 8 * 3]);
    let layer = Layer {
        raster: patch,
        x: 20,
        y: 30,
        quality: JxlQuality::Lossless,
    };
    let bytes = encode_layers(&base, &[layer], &lossless()).unwrap();
    assert_eq!(
        probe(&bytes).unwrap().frame_count,
        2,
        "stored as two layers"
    );
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded.frames.len(), 1, "displayed as one image");
    let px = decoded.into_first().to_rgba8().unwrap();
    assert_eq!(px.get_pixel(25, 33)[0], 200, "inside the layer");
    assert_eq!(px.get_pixel(5, 5)[0], 10, "outside the layer");
    assert_eq!(px.get_pixel(36, 33)[0], 10, "just right of the layer");
}

#[test]
fn animation_keeps_frames_and_timing() {
    let frames = vec![
        (photo(20, 20, 4), 100),
        (photo(20, 20, 4), 250),
        (photo(20, 20, 4), 50),
    ];
    let bytes = encode_animation(&frames, 0, &lossless()).unwrap();
    let img = decode(&bytes).unwrap();
    assert!(img.probe.is_animated);
    let durations: Vec<u32> = img.frames.iter().map(|f| f.duration_ms).collect();
    assert_eq!(durations, vec![100, 250, 50]);
}

#[test]
fn metadata_boxes_survive_and_policies_apply() {
    let mut img = photo(16, 16, 3);
    let exif = b"II*\0\x08\0\0\0\0\0\0\0\0\0".to_vec();
    img.metadata = Metadata {
        exif: Some(exif.clone()),
        xmp: Some(b"<x:xmpmeta/>".to_vec()),
    };

    let keep = encode(
        &img,
        &JxlEncodeOptions {
            metadata: MetadataPolicy::Keep,
            ..lossless()
        },
    )
    .unwrap();
    let back = decode(&keep).unwrap().into_first();
    assert_eq!(back.metadata.exif, Some(exif));
    assert_eq!(back.metadata.xmp.as_deref(), Some(&b"<x:xmpmeta/>"[..]));

    let strip = encode(
        &img,
        &JxlEncodeOptions {
            metadata: MetadataPolicy::StripAll,
            ..lossless()
        },
    )
    .unwrap();
    assert!(decode(&strip).unwrap().into_first().metadata.is_empty());
    assert!(strip.len() < keep.len());
}

#[test]
fn partial_file_still_shows_a_picture() {
    let img = photo(512, 512, 3);
    let opts = JxlEncodeOptions {
        progressive: true,
        effort: 3,
        focus: Some(FocusPoint { x: 400, y: 400 }),
        ..Default::default()
    };
    let bytes = encode(&img, &opts).unwrap();
    assert!(
        decode_prefix(&bytes[..20]).unwrap().is_none(),
        "too little data for a preview"
    );
    let half = decode_prefix(&bytes[..bytes.len() / 2])
        .unwrap()
        .expect("a preview at 50%");
    assert_eq!((half.width, half.height), (512, 512));
    let full = decode_prefix(&bytes).unwrap().expect("complete image");
    assert_eq!(full.samples, decode(&bytes).unwrap().into_first().samples);
}

#[test]
fn every_progressive_and_focus_combination_encodes() {
    // Regression test for the libjxl 0.12 progressive-DC + group-order bug.
    let img = photo(300, 300, 3);
    for quality in [JxlQuality::Lossless, JxlQuality::default()] {
        for progressive in [false, true] {
            for focus in [None, Some(FocusPoint { x: 250, y: 40 })] {
                let opts = JxlEncodeOptions {
                    quality,
                    progressive,
                    focus,
                    effort: 3,
                    ..Default::default()
                };
                assert!(encode(&img, &opts).is_ok(), "{opts:?}");
            }
        }
    }
}

#[test]
fn probe_reports_basic_properties() {
    let p = probe(&encode(&photo(30, 20, 4), &lossless()).unwrap()).unwrap();
    assert_eq!((p.width, p.height), (30, 20));
    assert!(p.has_alpha && p.uses_original_profile && !p.is_animated);
    assert_eq!(p.color_description, "sRGB");
}
