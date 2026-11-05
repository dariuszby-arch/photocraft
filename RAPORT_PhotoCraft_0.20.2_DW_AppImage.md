# PhotoCraft 0.20.2 DW 1 — AppImage i źródła

Data weryfikacji: 5 listopada 2026 r.

Nieoficjalne wydanie z rozszerzeniami Dariusza Wiśniewskiego, oparte na PhotoCraft
v0.20.2. Baza upstream: `7c12f647150f8e7b2e07718f7993b1fcaa57fefe`.
Migawka źródeł: `875520400b34651d2fbd0042b577fe192764b768`, drzewo Git:
`9fedb63a66c5c57452681a3fab85fb85eea38e60`.

## Zawartość

- AppImage: `PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage`, 70 687 224 bajty.
- Pełne źródła: `PhotoCraft_0.20.2_DW_Zrodla.zip`, z Cargo.lock, instrukcjami,
  licencjami i testami. Dodatkowy raport jest w katalogu DW_WERYFIKACJA.
- Program i CLI w AppImage są identyczne bajtowo z binariami zakończonej kompilacji.

Konwerter: **File → Export → Batch Converter…**. Można dodawać pojedynczy plik,
wiele plików albo katalog z filtrem i opcją podfolderów. Checkboxy wskazują pliki
do przetworzenia. Katalog wynikowy znajduje się pod listą. Jeden format i wspólne
ustawienia obowiązują dla całej uruchamianej kolejki. Dostępne są postęp, statusy
plików, zatrzymanie i podsumowanie błędów. JPEG XL działa bez zewnętrznych kodeków.

**Metadata: All** zachowuje dostępne EXIF i XMP. Dla zachowania lokalizacji
pozostaw wyłączone **Remove location info**. Metadata: None celowo usuwa metadane.
Domyślnie konwerter wybiera JXL, jakość 85 i Metadata: All. Włączenie Lossless
wybiera zapis bezstratny; przy zapisie stratnym jakość reguluje stopień kompresji.

## Wykonane sprawdzenia

| Sprawdzenie | Wynik |
|---|---|
| Integralność AppImage, rozpakowanie i uruchomienie dołączonego CLI | OK |
| Uruchomienie GUI z AppImage w Linux/X11, otwarcie JXL, zamknięcie z kodem 0 | OK |
| Bezstratny JXL: niezależne dekodowanie djxl i dokładna zgodność pikseli PNG | OK |
| Stratny JXL: niezależne dekodowanie djxl | OK |
| JPG → JXL → JPG, wariant bezstratny i stratny, 11 pól EXIF/XMP | OK |
| EXIF/XMP za obrazem w JXL, także po dodatkowym bloku 8 KiB | OK |
| Rzeczywisty moduł kolejki: JPG → JXL → JPG i niezależna kontrola 11 pól metadanych | OK |
| Pomijanie niezaznaczonego pliku i kontynuacja po uszkodzonym JPEG | OK |
| Metadata: None usuwa EXIF i XMP | OK |
| Tryb Overwrite nie nadpisuje źródeł, także niezaznaczonych | OK |
| Filtr katalogu, wielkość liter, podfoldery i usuwanie duplikatów | OK |
| Zatrzymanie kolejki i zachowanie oryginałów | OK |
| Render okna konwertera: 1366 × 768 i 1024 × 640; kontrola wizualna | OK |
| Spójność binariów i integralność CRC pełnego ZIP źródeł | OK |

Sprawdzone metadane: Make, Model, Orientation, ModifyDate, Artist, Copyright,
DateTimeOriginal, ISO, LensModel, XMP Creator i XMP Rights. Kontrola niezależna:
ExifTool 13.25 oraz djxl/libjxl 0.11.2. Testy kolejki wykorzystują bibliotekę
produkcyjną z tej samej kompilacji co aplikacja, przez publiczne polecenia menu.

## Kompilacja i wydajność

Rust 1.95.0, przypięte zależności Cargo.lock, profil dw-release, funkcja heif.
Kompilacja Linux x86_64 przez cargo-zigbuild z docelowym glibc 2.36.
Najwyższy wymagany symbol GLIBC w programie i CLI: **2.35**.
Paczka jest przeznaczona dla **MX Linux 23 x86_64** i nowszych zgodnych systemów.

Próbka syntetyczna 6000 × 4000 RGB: JPEG → stratny JXL, jakość 75 — **12,753 s**,
maksymalny RSS procesu około **801 MiB**. Źródło pozostało identyczne.
To prosty gradient; wynik nie przewiduje czasu ani rozmiaru dla dowolnej fotografii.

## Uruchamianie

Nadaj AppImage uprawnienie do wykonywania i uruchom dwuklikiem.
Przy braku FUSE można użyć:

```sh
APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
```

Przy problemie ze sterownikiem można dodać `--safe-gpu`.
Ustawienia tej paczki są oddzielone w `~/.config/PhotoCraftDW/0.20.2`.

## Zakres weryfikacji

Testy wykonano w Linux z programowym renderowaniem grafiki. Nie uruchamiano
tej paczki na fizycznym komputerze użytkownika z MX Linux. Nie wykonano pełnego
CI całego PhotoCraft ani wszystkich korpusów upstream. Ten raport dotyczy Linux;
nie potwierdza gotowości ani uruchomienia wydania Windows.

## Sumy SHA-256

AppImage:
`fe804c51cca063cddaf308bd50be32dda63e04852271d66e8b00138d3e6e457c`

Program w AppImage:
`730b128dd3f8b27488a9b687beb86284f18768dea8f088e36b9073f6eebdb7de`

CLI w AppImage:
`745d969d04e218aec8abba38403af54058fe11c7fad73da5fb54a5befa0e881f`
