# PhotoCraft 0.20.2 DW — rozbudowany eksport

Otwórz **File → Export → Export As**. Panel znajduje się po lewej stronie podglądu;
przy małym ekranie jego zawartość można przewijać.

- JPEG XL: **Lossless** oznacza zapis bezstratny. Wyłącz go, żeby ustawić **Quality** 1–100.
  Wyższa jakość zwykle oznacza większy plik. **Encoding effort** zmienia czas kodowania:
  Fast lub Normal. Dostępne jest też kodowanie progresywne.
- **Limit file size (KB)**: 0 wyłącza limit. Program dobiera jakość stratną na podstawie
  rzeczywistego pliku w docelowych wymiarach, maksymalnie do wybranej jakości.
  Jeśli limit jest nieosiągalny albo plik bezstratny jest za duży, eksport zwraca błąd
  przed zapisem. Nie zmienia sam rozdzielczości ani trybu bezstratnego.
- Format: JPEG, PNG, TIFF, PSD, WebP, AVIF, JPEG XL, DNG, EXR, Radiance HDR, TGA, ICO
  i niezmieniona kopia pliku źródłowego (**Original file**, tylko zapisany dokument).
- Przestrzenie barw: profil dokumentu, sRGB, Display P3, Adobe RGB, ProPhoto i Rec.2020.
  Kompresja stratna JXL używa sRGB; JXL HDR używa Rec.2020 PQ, 16 bitów, biel SDR 203 nitów.
  HDR dla TIFF/EXR/Radiance zapisuje liniowe sRGB z danymi zmiennoprzecinkowymi.
  Opcja HDR jest aktywna dla formatów z takim zapisem. Nie odtwarza utraconych świateł zdjęcia SDR.
- **Full size**, dłuższy/krótszy bok, szerokość, wysokość, pole wymiarów, megapiksele,
  procent lub dokładne wymiary; **Don't enlarge**, metoda skalowania i rozdzielczość ppi.
- Wyostrzenie wyjściowe: ekran, papier matowy lub błyszczący oraz trzy poziomy siły.
- Metadane: wszystkie, bez danych aparatu, autor/prawa autorskie lub żadne;
  osobna opcja usunięcia lokalizacji. Rozdzielczość ppi jest zapisywana niezależnie,
  jeśli wybrany format ją obsługuje.
- Tekstowy znak wodny: treść (także znacznik {name}), rozmiar, krycie, kolor i dziewięć pozycji.
- Nazwa pliku ze znacznikami widocznymi pod **Tags…**, folder docelowy, podfolder
  i zachowanie przy istniejącym pliku: numerowanie, nadpisanie lub pominięcie.
- Presety zapisują ustawienia całego panelu. Starsze presety i nagrane akcje pozostają obsługiwane.

Zmiany dotyczą kopii eksportowej. Otwarty dokument pozostaje edytowalny.
Wyostrzanie wyjściowe spłaszcza kopię; formaty płaskie, w tym JXL, nie przechowują warstw.
DNG jest wywołanym, liniowym obrazem RGB 16 bitów bez kompresji (z wybranym XMP),
nie kopią surowych próbek oryginalnego aparatu. **Original file** kopiuje źródło bez zmian
(wraz z jego metadanymi); kontrolki przetwarzania są wtedy wyłączone.

AppImage: Linux x86_64. Nadaj prawo uruchamiania w jego właściwościach albo:

```sh
chmod +x PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
```

Bez FUSE:

```sh
APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
```

Kodeki są wbudowane; instalacja cjxl/djxl/libjxl nie jest potrzebna.
**File → Save As → JPEG XL** nadal służy do bezstratnego zapisu z domyślnymi ustawieniami.

## Zachowanie EXIF i XMP

Przy konwersjach JPG → JXL i JXL → JPG wybierz **Metadata: All**.
Dla zachowania również danych lokalizacji wyłącz **Remove location info**.
Poprawiono odczyt metadanych umieszczonych po obrazie w kontenerze JXL.
Bieżące wyniki weryfikacji gotowej paczki znajdują się w `RAPORT_PhotoCraft_0.20.2_DW_AppImage.md`.
Zakres wykonanej weryfikacji opisano w raporcie dołączonym do paczki.

## Konwersja wielu zdjęć

**File → Export → Batch Converter…** otwiera kolejkę plików z checkboxami, filtrem katalogu,
katalogiem wynikowym i postępem konwersji. Jeden format obowiązuje dla całej kolejki.
Szczegóły: `BATCH_CONVERTER_PL.md`.
