# PhotoCraft 0.20.2 DW 1

Unofficial community edition by Dariusz Wiśniewski, based on upstream v0.20.2.

## Changes

- Full JPEG XL read/write support with lossless/lossy selection, quality,
  encoding effort, progressive encoding and EXIF/XMP metadata.
- Integrated Batch Converter: files or filtered folders, optional subfolders,
  checkboxes, one selected output format per queue, output folder, progress,
  cancellation and per-file results. Original inputs remain protected.
- Extended Export As options and metadata-policy controls.
- Metadata after the JXL codestream is read correctly.

Menu: **File → Export → Batch Converter…**.
For EXIF/XMP preservation, use **Metadata: All** with **Remove location info** off.

## Linux build and verification

AppImage: `PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage` (70,687,224 bytes).
Built with Rust 1.95.0, locked dependencies, profile dw-release and heif enabled,
using cargo-zigbuild with glibc 2.36 as the target. Includes the GUI and CLI.

Verified: AppImage launch under Linux/X11, independent libjxl/djxl decoding,
exact pixels in lossless PNG → JXL, lossy decoding, and 11 EXIF/XMP fields in
JPG → JXL → JPG (including trailing metadata boxes). Production-library queue
checks cover unchecked files, bad input, folder masks/recursion/deduplication,
cancellation, Metadata: None, and source overwrite protection. Converter UI
rendered and visually checked at 1366×768 and 1024×640.

See [the complete report](RAPORT_PhotoCraft_0.20.2_DW_AppImage.md).
These are targeted checks, not a full upstream CI/corpus run or a test on the
user's physical MX Linux system. Windows runtime verification is not included.

AppImage SHA-256:
`fe804c51cca063cddaf308bd50be32dda63e04852271d66e8b00138d3e6e457c`

Build source commit: `875520400b34651d2fbd0042b577fe192764b768`.
Subsequent publication commits add documentation and acceptance-test evidence.

## Run

```sh
chmod +x PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
```

Without FUSE:

```sh
APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
```

Use `--safe-gpu` for the CPU compatibility path, or `--cli` for the bundled CLI.
Settings are isolated under `~/.config/PhotoCraftDW/0.20.2`.

PhotoCraft and these extensions retain their existing MIT OR Apache-2.0 licences.
Developed with AI-assisted programming tools.
