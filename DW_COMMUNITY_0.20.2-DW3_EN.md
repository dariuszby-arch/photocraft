# PhotoCraft 0.20.2-DW3 — PSD regression fix, JPEG XL and Batch Converter

**20 November 2026 · Unofficial community fork based on PhotoCraft 0.20.2**

I would like to share the results of my work on PhotoCraft and invite independent testing and technical discussion. My goal is to contribute useful fixes and improvements to the original project.

## 1. PSD import regression (FIX3)

PhotoCraft 0.17.2 could open my original PSD document (1491 × 1055 pixels, RGB/8, 18 layers). Newer builds refused to open the same file on my MX Linux machine with 32 GB of RAM and about 20 GiB available.

The exact error was:

> its estimated memory requirement of 79598444 bytes cannot be admitted within the 31326208 byte hard limit on this machine

The import needed about 76 MiB, but the application reported a hard limit of roughly 30 MiB.

### Root cause and fix

In the affected legacy cgroups v1/cgmanager environment, the process belonged to the memory hierarchy, but its mount view exposed only the name=systemd hierarchy. The memory detector conflated an unexposed memory controller with actual zero available RAM (Some(0)). This caused the importer to refuse a valid document before decoding. The small nonzero GUI limit came from a scratch-tile credit, not the machine's physical RAM.

FIX3 distinguishes an unexposed controller (CgroupLimits::Unexposed) from real zero memory and passes an unknown value (None) to the existing bounded fallback import policy. This policy has a 2 GiB automatic-import threshold; it is not a claim that the machine has 2 GiB RAM. Readable limits and genuinely zero memory remain enforced. Unreadable exposed interfaces and malformed data remain conservatively handled.

**Result:** the original PSD opens successfully again on the affected MX Linux machine, with 18 layers. This is a confirmed regression in that environment, not a claim that every newer build or OS is affected.

### Reproduction and tests

A public synthetic test document, [PSD_memory_regression_18_layers.psd](https://github.com/dariuszby-arch/photocraft/releases/download/v0.20.2-dw3/PSD_memory_regression_18_layers.psd), is included in the release. It is 1491 × 1055 px, RGB/8, 18 layers, ZIP-compressed, approximately 224 KiB on disk. It is a separate test fixture with different pixel data from my private original PSD.

The technical report records 19 passing memory detector tests, 20 import-policy tests, 25 PSD memory-budget tests and 5 JPEG XL/metadata tests. The synthetic document reproduces the same memory-detection failure in the reconstructed affected environment and opens with all 18 layers after FIX3. Both published packages passed the release workflow's CLI import and JPEG XL encode/decode checks. This is targeted verification, not exhaustive GUI testing.

- [Technical regression report and reproduction](https://github.com/dariuszby-arch/photocraft/blob/v0.20.2-dw3/PSD_MEMORY_REGRESSION_PL.md)
- [Memory detector source](https://github.com/dariuszby-arch/photocraft/blob/v0.20.2-dw3/crates/raster/src/memory/system.rs)
- [Import memory policy](https://github.com/dariuszby-arch/photocraft/blob/v0.20.2-dw3/crates/io/src/memory.rs)

## 2. Extended JPEG XL workflow

The DW fork retains extended JPEG XL reading and writing, based in part on the LightCraft implementation, with lossless and lossy encoding, quality controls, Fast and Normal/smaller-file effort modes, progressive encoding and HDR options. The standard Export As dialog also exposes transparency, colour-profile embedding, resizing, sharpening, preview and estimated output size.

Metadata handling has been improved for supported JPEG/JPEG XL workflows: EXIF and XMP preservation, standard JPEG XL EXIF container interoperability and ICC profile handling. Select Metadata: All, and disable Remove location info if GPS data should be retained. Lossless encoding preserves image samples when no pixel-changing export transformations are applied; it does not preserve an editable PSD layer stack in JXL.

## 3. Integrated Batch Converter

Open **File → Export → Batch Converter…**.

Add individual files, multiple files or a directory, filter by extension and optionally include subdirectories. Select the files to process, choose one target format for the entire queue, set shared export parameters and select the destination folder. The converter displays progress and supports cancellation while protecting source files.

For example, a queue containing JPEG and PSD inputs can be exported to JPEG XL with the same chosen settings. This is an image conversion workflow rather than a way to preserve PSD layers in a JXL image.

## 4. Windows build under Wine

I also tested the Windows portable build on MX Linux through Wine. Automatic WGPU backend selection initially failed with:

> WGPU error: Parent device is lost

The following workaround allowed the application to start and open PSD files successfully:

```sh
WGPU_BACKEND=vulkan wine ./photocraft.exe
```

The failing automatic backend has not been established; automatic Wine detection and Vulkan preference are possible future improvements, not features claimed for DW3.

## 5. Downloads

[PhotoCraft 0.20.2-DW3 release](https://github.com/dariuszby-arch/photocraft/releases/tag/v0.20.2-dw3)

Available assets: Windows x64 portable ZIP, MX Linux 23-compatible x86_64 AppImage, synthetic PSD regression fixture, SHA-256 checksums and tagged source archives. Binaries are unsigned. Native builds include both the GUI and CLI.

## 6. Screenshot captions

The accompanying original screenshot set illustrates the following (image files are not embedded in this document):

1. **Before FIX3:** PSD rejected with an incorrect hard limit of 31,326,208 bytes.
2. **After FIX3:** same original PSD opens, with 1491 × 1055 px and 18 layers shown in the status bar.
3. **Menu integration:** File → Export → Batch Converter.
4. **Batch conversion:** queue containing JPEG and PSD inputs, JPEG XL selected as the target format; conversion is awaiting start in the screenshot.
5. **JPEG XL export:** quality at 95%, Normal/smaller-file effort and progressive JPEG XL enabled; displayed preview and estimated file size.

---

This is an unofficial fork, not an official upstream release. Credit and licensing notices for the original project and third-party work are retained. I appreciate the work of the PhotoCraft developers and welcome feedback, independent testing and potential upstream integration of the fix.

[Original PhotoCraft project](https://github.com/storytold/photocraft) · [DW fork](https://github.com/dariuszby-arch/photocraft)
