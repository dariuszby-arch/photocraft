# PhotoCraft 0.20.2 DW 2 — JPEG XL + Batch Converter

Wersja społecznościowa / Community edition. Windows x64 portable + Linux x86_64 AppImage.

- Preserves EXIF/XMP during JPG/JXL conversion; reconciles known SDR colour tags.
- Preserves source ICC in lossless JXL; converts P3 correctly when ICC embedding is disabled.
- Retains pixel tiles when source and export profiles match.
- JPEG XL lossless/lossy, quality, Fast/Normal effort, progressive, HDR and batch conversion.
- Additional malformed-EXIF regression coverage and CI lint corrections.

Windows: unzip and run photocraft.exe. Linux/MX 23: chmod +x the AppImage and launch it.
Binaries are unsigned. Verify SHA256SUMS.txt. This is not an official upstream release.
The previous DW 1 release is retained.

Build verification: the publication workflow checks both executables and performs a JPEG XL round trip before uploading assets. Download files only from the completed DW 2 release.
