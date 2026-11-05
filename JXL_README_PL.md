# PhotoCraft 0.20.2 DW — JPEG XL

Otwieranie: **File → Open**, filtr JPEG XL, albo przeciągnięcie pliku `.jxl`.
Zapis: **File → Save As → JPEG XL**. Ustawienia jakości, kompresji i metadanych:
**File → Export → Export As** lub **File → Export → Batch Converter…**.

- **Lossless**: kompresja bezstratna. Po wyłączeniu działa **Quality** 1–100.
- **Encoding effort**: Fast albo Normal. Dostępne jest kodowanie progresywne.
- Odczyt plików z kontenerem i bez kontenera; zapis RGB/RGBA 8/16-bit.
- Szarości przy rozszerzonym zapisie są rozwijane do RGB; alfa jest zachowywana.
- Bezstratny eksport może osadzać profil RGB ICC. Stratny używa sRGB.
- **Metadata: All** zachowuje dostępne EXIF/XMP; **Remove location info**
  celowo usuwa lokalizację. Źródło musi zawierać te metadane.
- JXL jest formatem płaskim. Edytowalne warstwy zachowuj w PSD lub `.pcraft`.

Kodeki są wbudowane; `cjxl`, `djxl` i systemowe libjxl nie są wymagane.
Domyślna funkcja Cargo `jxl` włącza mechanizm przeniesiony z LightCraft.
Podstawowy bezstratny enkoder upstreamu 0.20.2 pozostał jako wariant dla
kompilacji bez tej funkcji i dla WebAssembly.

Instrukcja obu systemów i kompilacji: `DW_README_PL.md`.
Pochodzenie i licencje: `NOTICE-JXL.md` oraz `third-party/`.
Zakres testów konkretnej paczki znajduje się w dołączonym raporcie.
