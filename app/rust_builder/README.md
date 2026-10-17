# jixel_native

A Flutter FFI plugin whose only job is to build the Rust crate in
[`../rust`](../rust) for the current platform and bundle it with the app.
The heavy lifting is done by [Cargokit](https://github.com/irondash/cargokit)
(vendored in `cargokit/`, MIT/Apache-2.0).

| Platform | Artifact | Wiring |
|---|---|---|
| iOS, macOS | `libjixel_bridge.a`, force-loaded into the plugin framework | `*/jixel_native.podspec` |
| Android | `libjixel_bridge.so` per ABI | `android/build.gradle` |
| Linux, Windows | `libjixel_bridge.so` / `jixel_bridge.dll` | `*/CMakeLists.txt` |

You need [rustup](https://rustup.rs) installed; Cargokit adds the required
Rust targets automatically. See `docs/en/building.md`.
