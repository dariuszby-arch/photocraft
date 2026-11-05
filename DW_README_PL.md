# PhotoCraft 0.20.2 — rozszerzenia Dariusza Wiśniewskiego

Nieoficjalna wersja oparta na tagu upstream **v0.20.2**
(`7c12f647150f8e7b2e07718f7993b1fcaa57fefe`).
Nie zastępuje oryginalnego projektu ani jego autorstwa.

## Rozszerzenia

- **File → Export → Batch Converter…**: pliki pojedyncze, wiele plików i foldery,
  maski nazw, podfoldery, zaznaczenia, jeden format dla całej kolejki, katalog
  wynikowy, postęp i zatrzymywanie. Źródła są chronione przed nadpisaniem.
- **JPEG XL**: odczyt i zapis, kompresja bezstratna i stratna, jakość,
  wysiłek kodowania, zapis progresywny, RGB/RGBA 8/16-bit i metadane EXIF/XMP.
- **Export As**: limit rozmiaru pliku, skalowanie, rozdzielczość, wyostrzenie,
  znak wodny, wybór profilu, polityka metadanych, presety, nazewnictwo i katalog.
- Poprawiony odczyt EXIF/XMP także wtedy, gdy bloki metadanych znajdują się za
  obrazem w kontenerze JXL.

Kodeki są wbudowane. Do JPEG XL nie potrzeba `cjxl`, `djxl` ani systemowego libjxl.
Kompresja stratna JXL korzysta z sRGB; bezstratna może zachować osadzony profil RGB.
Szarości w rozszerzonym enkoderze są rozwijane do RGB. JXL zapisuje obraz płaski,
nie edytowalne warstwy. Dostępne opcje nie odtwarzają szczegółów utraconych w źródle.

Dla zachowania metadanych wybierz **Metadata: All**. Zaznaczone **Remove location
info** celowo usuwa dane lokalizacji. Metadane nieobecne w źródle nie są odtwarzane.

## Linux x86_64

Paczka AppImage jest przeznaczona dla systemów z glibc **2.36 lub nowszą**, w tym
MX Linux 23. Wymaga działającej sesji graficznej i obsługi OpenGL lub Vulkan.

Nadaj AppImage prawo uruchamiania i uruchom go. Bez FUSE:

```sh
APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage
```

Ta paczka AppImage przechowuje ustawienia osobno, w katalogu
`~/.config/PhotoCraftDW/0.20.2` (chyba że ustawiono `PHOTOCRAFT_CONFIG_DIR`).

W razie problemu ze sterownikiem można dodać `--safe-gpu`. Opcja `--cli`
uruchamia dołączony interfejs wiersza poleceń.

## Windows x64

Rozpakuj całą paczkę do zapisywalnego folderu i uruchom **photocraft.exe**.
Nie przenoś samego EXE, jeśli obok znajdują się biblioteki DLL.
Dołączony `portable.txt` włącza standardowy tryb przenośny PhotoCraft; ustawienia
i pliki odzyskiwania trafiają do `PhotoCraftData` obok programu.

To wydanie zawiera własne rozszerzenia. Zainstalowanie oficjalnej aktualizacji
z poziomu programu zastąpi je oficjalną wersją; kolejne wydania DW należy pobierać
z gałęzi lub wydań autora rozszerzeń.

## Źródła i odtwarzanie kompilacji

Repozytorium: <https://github.com/dariuszby-arch/photocraft>.
Pracuj z gałęzią integracji pełnej wersji 0.20.2, a nie ze starszą, częściową
gałęzią EXIF/XMP. Wymagany Rust **1.95+** i zależności z `Cargo.lock`.

Profil `dw-release` zachowuje zoptymalizowane przetwarzanie obrazów i ogranicza
zużycie pamięci kompilatora w dużych modułach interfejsu oraz poleceń. Można go
wybrać zamiast `--release`, podając `--profile dw-release`; wtedy wyniki trafiają
do katalogu `dw-release`, a nie `release`.

Linux:

```sh
cargo build --release --locked -p photocraft -p photocraft-cli --features heif --bins
```

Na MX Linux zainstaluj narzędzia wymienione w `docs/development.md`.
Do budowania AppImage służy `packaging/linux/package.sh`; ustaw
`GITHUB_REPOSITORY=dariuszby-arch/photocraft`, aby metadane aktualizacji
samodzielnie budowanej paczki odnosiły się do własnego forka.

Windows (natywnie, Rust MSVC i Visual Studio C++ Build Tools):

```powershell
cargo build --release --locked --target x86_64-pc-windows-msvc -p photocraft -p photocraft-cli --features heif --bins
```

Wynik jest w `target/release` albo `target/<target>/release`.
Dołączone binaria mogą być budowane krzyżowo; szczegóły konkretnej paczki i zakres
wykonanych testów zapisano w raporcie dostarczanym z wydaniem.

## Pochodzenie i licencje

Rozszerzenia przeniesiono z pełnej zmodyfikowanej migawki 0.17.2 autora. Zachowano
nowsze mechanizmy 0.20.2, m.in. przygotowanie eksportu, obsługę profili, kontrolę
pamięci i wcześniejsze testy próbek 16-bitowych z przezroczystością.
Podstawowy enkoder JXL z 0.20.2 pozostaje jako wariant bez domyślnej funkcji
`jxl` oraz dla WebAssembly. Pełny konwerter wsadowy jest funkcją wersji desktopowej.

PhotoCraft oraz rozszerzenia pozostają na licencji MIT OR Apache-2.0. Mechanizm
JXL pochodzi z LightCraft; zobacz `NOTICE-JXL.md` i `third-party/`.
Program rozwijano z użyciem narzędzi wspomagających programowanie przez AI.
