# JPEG XL port from LightCraft

The bounded-feed decoder and jixel encoder mechanism in `crates/codecs/src/codecs/jxl.rs`
and export settings in `crates/codecs/src/jxl_options.rs` are adapted from the supplied
LightCraft `crates/codecs/src/jxl.rs` and `jxl_encode.rs`.

Copyright (c) 2026 ArtCraft Team and the LightCraft contributors.
Licensed under MIT OR Apache-2.0. Original notices and licences are retained in
`third-party/lightcraft/` (including the complete upstream NOTICE for attribution).
Only the JXL mechanism was ported; LightCraft's other features are not bundled.

Codec dependencies: jxl-oxide 0.12.6 (MIT OR Apache-2.0),
jixel 0.3.4 (BSD-3-Clause OR Apache-2.0).
Lossless export supports 8/16-bit RGB/RGBA (gray is expanded); other source depths
are converted with PhotoCraft's existing fidelity warnings. Lossy JXL uses sRGB;
the document export path converts through the source ICC profile first.
