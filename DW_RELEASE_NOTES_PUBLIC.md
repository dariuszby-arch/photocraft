# PhotoCraft 0.20.2 DW 1 — Windows x64 + Linux x86_64

**Nieoficjalna wersja społecznościowa PhotoCraft przygotowana przez Dariusza Wiśniewskiego.** Bazuje na PhotoCraft 0.20.2 i zawiera autorskie rozszerzenia. Nie jest wydaniem oficjalnego zespołu PhotoCraft.

## Pobieranie / Downloads

- **Windows 10/11, 64-bit:** `PhotoCraft_0.20.2_DW_1_Windows_x64_Portable.zip` — przenośny program `photocraft.exe` oraz `photocraft-cli.exe`. Rozpakuj ZIP i uruchom `photocraft.exe`; nie wymaga instalacji.
- **Linux x86_64 (MX Linux 23 i zgodne):** `PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage` — po pobraniu nadaj prawo wykonywania i uruchom. Bez FUSE: `APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage`.
- **Weryfikacja:** `SHA256SUMS.txt` zawiera kontrolne sumy SHA-256 obu pakietów.
- **Kod źródłowy:** archiwa „Source code (zip)” i „Source code (tar.gz)” są tworzone przez GitHub z taga wydania.

## Najważniejsze zmiany / Highlights

- **JPEG XL:** odczyt i zapis JXL, kompresja bezstratna i stratna, regulacja jakości i wysiłku kodowania, zapis progresywny.
- **EXIF/XMP:** zachowanie obsługiwanych metadanych w przepływie JPG ↔ JXL, w tym kontenerów JXL z metadanymi za obrazem. Wybierz **Metadata: All**, a aby zachować GPS, wyłącz **Remove location info**.
- **Batch Converter:** kolejka pojedynczych plików, wielu zdjęć i katalogów z maskami oraz opcjonalnymi podkatalogami. Checkboxy, jeden format na kolejkę, folder wynikowy, postęp, zatrzymanie i ochrona oryginałów.
- **Rozszerzony eksport:** skalowanie, wyostrzenie, znak wodny, profile kolorów, polityka metadanych i ustawienia wyników.

**Gdzie znaleźć konwerter?** `File → Export → Batch Converter…`

## Informacje o testach / Verification

Użytkownik potwierdził działanie swoich rozszerzeń w kompilacji Windows x64. Wcześniejszy raport projektu opisuje ukierunkowane testy JPEG XL, konwersji i 11 pól EXIF/XMP w Linux. Pakiety tej publikacji są budowane i sprawdzane przez GitHub Actions; nie oznacza to wykonania pełnego zestawu testów upstream ani testu Linux na fizycznym MX Linux użytkownika.

**Uwaga:** kompilacja Windows jest bez podpisu cyfrowego. Windows SmartScreen może wyświetlić ostrzeżenie.

---

# English

**Unofficial community build by Dariusz Wiśniewski**, based on PhotoCraft 0.20.2; not an official upstream release.

Includes extended JPEG XL read/write (lossless, lossy, quality, effort, progressive), EXIF/XMP preservation including trailing JXL metadata, and a full Batch Converter with file/folder selection, filters, one target format per queue, progress, cancellation and source-file protection. The export dialog also exposes additional output controls.

**Menu:** `File → Export → Batch Converter…`.

**Windows:** extract the portable ZIP and run `photocraft.exe` (unsigned). **Linux:** run the x86_64 AppImage (MX Linux 23-compatible target). See the provided `SHA256SUMS.txt` for checksums. Windows features were user-tested; full upstream CI and physical-machine Linux validation are not claimed.

**Licensing:** MIT OR Apache-2.0; see bundled notices and upstream credits.
