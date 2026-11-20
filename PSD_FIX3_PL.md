# PhotoCraft 0.20.2 DW — PSD FIX3 (MX / cgmanager)

Data: 20.11.2026. Baza DW: `9ddbcd336ef67516b5bc78c420e8098e3dacc412`.
Zachowano pełną bazę DW, JPEG XL, EXIF/XMP i konwerter wsadowy.
Numer programu zawiera `0.20.2-DW-PSD-FIX3`.

## Ustalona przyczyna

Raport z MX pokazał:

- `MemTotal: 32754536 kB` — około 31,2 GiB RAM dostępnego dla systemu;
- `MemAvailable: 20800996 kB` — około 19,8 GiB dostępnego RAM;
- przynależność procesu do hierarchii cgroups v1 `memory`;
- w widoku montowań procesu tylko cgroup `name=systemd`, zarządzaną przez cgmanager,
  bez zamontowanego kontrolera `memory`.

FIX2 uznawał ten stan za `Some(0)` dostępnego RAM. Przy aktywnym scratch import
otrzymywał tylko 31 326 208 bajtów kredytu na kafelki, a wymagał 79 598 444 bajtów.
W źródłach 0.17.2 tej adaptacyjnej bramki importu jeszcze nie było, dlatego
wcześniejsza wersja nie odrzucała pliku w tym miejscu. Jest to regresja nowszego
mechanizmu wykrywania pamięci, niezależna od liczby zainstalowanych modułów RAM.

FIX1 i FIX2 obsługiwały inne sytuacje (brak montowań i puste członkostwo), ale nie
układ kontrolerów widoczny w raporcie z MX.

## Co zmienia FIX3

Gdy kompletne informacje o montowaniach pokazują wyłącznie niemające związku
z pamięcią hierarchie v1, a członkostwo procesu wskazuje dodatkowy kontroler
`memory`, jego limit jest **nieznany**, a nie równy zeru.

Importer używa wtedy swojej istniejącej polityki dla nieznanej pojemności:
automatyczne przyjęcie do 2 GiB oszacowanej pamięci na import, z dotychczasową
ścieżką potwierdzania większych importów. 2 GiB jest progiem automatycznego
importu, nie odczytem ilości RAM w komputerze. Ten PSD wymaga około 76 MiB,
więc mieści się w tym progu.

Nie zakładamy, że nieodczytywalny kontroler ma nieograniczony limit. Odczytywalne
limity cgroups nadal obowiązują. Widoczny, ale nieodczytywalny interfejs pamięci
oraz uszkodzone informacje o montowaniach nadal są traktowane ostrożnie.
Rzeczywiste zero dostępnego RAM pozostaje zerem.

W raporcie diagnostycznym `available_ram: None` wraz z objaśnieniem oznacza
nieznany limit procesu, a nie brak fizycznej pamięci. Rzeczywiste dane RAM
systemu pozostają widoczne w sekcji `/proc/meminfo`.

## Potwierdzone sprawdzenia

- FIX2 w widoku odtworzonym z raportu MX zwraca dokładnie `Some(0)` oraz
  `constraint_unreadable: true`; nowy kod rozróżnia nieznany limit i zero.
- Testy dostawcy pamięci: 19/19. Test obejmuje układ cgmanager, widoczne limity,
  uszkodzone wpisy oraz rzeczywiste zero dostępnego RAM.
- Pełne testy rastra: 84 poprawne, 1 pominięty (uruchomienie szeregowe).
- Polityka pamięci importu: 20/20, w tym dopuszczenie raportowanego PSD przy
  nieznanym limicie cgmanager i odmowa przy rzeczywistym zerze RAM.
- Testy JPEG XL / EXIF/XMP / 16 bitów / ICC: 5/5.
- Kontrola warstw zależności: 31 pakietów bez naruszeń. WebAssembly: 24 pakiety
  i dodatkowy wariant kodeków HEIF — poprawne.
- Nowy CLI, przy układzie kontrolerów odtworzonym z raportu MX i glibc 2.36,
  otwiera dostarczony PSD: 1491 × 1055, wszystkie 18 warstw.
- W tych samych warunkach PSD → PNG i PSD → JXL lossless → PNG dają identyczne
  piksele. Eksport i ponowny odczyt stratnego JXL również działają.
- Oba programy zgłaszają `0.20.2-DW-PSD-FIX3`; najwyższy wymagany przez GUI
  symbol glibc to 2.35 (MX 23 ma glibc 2.36).
- Clippy pakietów raster i IO, bez zależności, wszystkie cele — poprawny;
  formatowanie zmienionych plików — poprawne.
- Gotowy AppImage: ten sam układ MX/cgmanager, odczyt 18 warstw poprawny.
  Pliki wykonywalne z obrazu mają identyczne skróty jak sprawdzona kompilacja.

Poprawka została również potwierdzona przez użytkownika na jego rzeczywistym
komputerze MX Linux z 32 GB RAM: PSD otwiera się prawidłowo.

Opis do zgłoszenia i publiczny syntetyczny plik PSD: [PSD_MEMORY_REGRESSION_PL.md](PSD_MEMORY_REGRESSION_PL.md).

## Uruchomienie

```bash
chmod +x PhotoCraft_0.20.2_DW_PSD_FIX3_MX23-x86_64.AppImage
./PhotoCraft_0.20.2_DW_PSD_FIX3_MX23-x86_64.AppImage
```

Bez FUSE:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_PSD_FIX3_MX23-x86_64.AppImage
```

Diagnostyka tylko do odczytu:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_PSD_FIX3_MX23-x86_64.AppImage --memory-report
```

Dodatkowy `sprawdz_PSD_FIX3.sh` uruchomiony obok AppImage pokaże wersję i odczyt
pamięci. Opcjonalnie przyjmuje ścieżkę PSD i sprawdza jego import bez zapisu:

```bash
bash sprawdz_PSD_FIX3.sh '/pełna/ścieżka/do/pliku.psd'
```

## Budowanie

```bash
bash packaging/linux/buduj_DW_PSD_FIX3.sh
```

Wymagany jest Rust co najmniej z `Cargo.toml`, narzędzia kompilacji systemowej,
curl i tar oraz dostęp do przypiętych zależności. Skrypt nie publikuje niczego
w GitHubie. Dokumentacja wcześniejszych FIX opisuje historię; właściwym
punktem wejścia dla tego wydania jest skrypt FIX3.

Prywatny PSD użytkownika nie jest dołączany do paczki źródłowej. Licencje i
informacje o autorach pozostają w LICENSE, NOTICE, NOTICE-JXL.md, ATTRIBUTION.md
i third-party/lightcraft.
