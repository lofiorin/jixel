# Models

## u2netp.onnx

| | |
|---|---|
| Model | U²-Netp (the 1.1 M-parameter "small" U²-Net) |
| Task | salient object segmentation → background removal and subject detection |
| Input | 1×3×320×320, ImageNet-normalised RGB |
| Output | 1×1×320×320 saliency (first of 7 side outputs) |
| Size | 4 574 861 bytes |
| SHA-256 | `309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8` |
| Licence | Apache-2.0 |
| Authors | Xuebin Qin, Zichen Zhang, Chenyang Huang, Masood Dehghan, Osmar R. Zaiane, Martin Jagersand |
| Paper | *U²-Net: Going Deeper with Nested U-Structure for Salient Object Detection*, Pattern Recognition 106, 2020 |
| Source | ONNX export published with [rembg](https://github.com/danielgatis/rembg) (release `v0.0.0`, `u2netp.onnx`), from the original weights at [xuebinqin/U-2-Net](https://github.com/xuebinqin/U-2-Net) |

It is embedded into the binary at compile time (`include_bytes!` in
`core/crates/jixel-core/src/ops/background.rs`), so the app works offline and
never downloads anything.

**Why this model:** it is small enough for any phone or PC (4.6 MB, runs at
320×320 in well under a second on a phone CPU), permissively licensed, and
good at "one main subject" photos — products, people, pets. Jixel improves its
low-resolution edges with a guided filter at full resolution.

**Verify:**

```bash
shasum -a 256 models/u2netp.onnx
```

**Replacing it:** any ONNX model with the same input/output shapes works.
Update the hash above and the notice in `THIRD_PARTY_NOTICES.md`.
