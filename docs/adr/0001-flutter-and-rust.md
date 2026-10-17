# ADR 0001: Flutter for the UI, Rust for the engine

- Status: accepted
- Date: 2026-10-17

## Context

Jixel must run on iOS, Android, macOS, Windows and Linux with one codebase,
look and feel like a modern native app, and do heavy image work (libjxl,
AVIF/WebP encoders, neural network inference) fast and safely.

## Decision

- **UI: Flutter.** One codebase for five platforms, Material 3, excellent
  desktop support (drag and drop, adaptive layouts), mature tooling.
- **Engine: Rust** (`core/`), wrapping libjxl (C++) and using pure-Rust crates
  for everything else. The same engine powers the CLI.

## Alternatives considered

- *Kotlin/Compose Multiplatform*: iOS and desktop less mature for this use.
- *Tauri/Electron + web UI*: no mobile (Tauri mobile still young), heavier.
- *Native apps per platform*: five UIs to maintain.
- *Engine in Dart*: no mature JPEG XL encoder; much slower image code.
- *Engine in C++*: memory safety and build ergonomics; Rust has the
  codec/ML ecosystem we need (`image`, `ravif`, `tract`, `ssimulacra2`).

## Consequences

- Contributors need Rust and Flutter toolchains (documented in building.md).
- Engine logic is testable and reusable without any UI.
