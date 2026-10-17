# ADR 0002: Hand-written libjxl bindings

- Status: accepted
- Date: 2026-10-17

## Context

Jixel needs libjxl's encoder and decoder from Rust, including less common APIs:
frame (layer) settings, group order and centre, progressive passes, JPEG
reconstruction, boxes, streaming partial input.

The existing `jpegxl-sys` / `jpegxl-rs` crates are licensed **GPL-3.0**, which
is incompatible with distributing Jixel under Apache-2.0. Pure-Rust
alternatives (`jxl-oxide`, `jxl-rs`) are decoders only.

## Decision

`jixel-libjxl-sys` builds libjxl 0.12 from a git submodule with CMake and
declares, by hand, only the functions, enums and structs we use (with doc
comments pointing to the libjxl headers). Safe wrappers live in
`jixel-core::codec::jxl`.

## Consequences

- We control exactly which libjxl version and options are built, on every
  platform, including iOS and Android cross-compilation.
- Updating libjxl means checking the declarations against the new headers.
  The round-trip tests catch ABI mistakes quickly.
- libjxl is BSD-3-Clause: compatible with Apache-2.0.
