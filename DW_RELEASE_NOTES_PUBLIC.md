# PhotoCraft 0.20.2 DW 3 — PSD FIX3 + JPEG XL

Społecznościowe wydanie forka DW / Unofficial community fork release.
W programie i w CLI numer wersji to **0.20.2-DW3**.

## Pobieranie i uruchamianie

| System | Plik | Uruchomienie |
| --- | --- | --- |
| MX Linux 23 i zgodne dystrybucje, x86_64 | `PhotoCraft_0.20.2_DW_3_MX23-x86_64.AppImage` | Nadaj prawo wykonywania (`chmod +x`) i uruchom AppImage. |
| Windows 10/11, x64 | `PhotoCraft_0.20.2_DW_3_Windows_x64_Portable.zip` | Rozpakuj ZIP do zapisywalnego katalogu i uruchom `photocraft.exe`. |

Pakiety zawierają edytor oraz CLI. Linux wymaga glibc 2.35 lub nowszej i działającego sterownika graficznego. Wersja Windows działa w trybie portable. Pliki nie są podpisane cyfrowo; sumy kontrolne obu pakietów i przykładu PSD znajdują się w `SHA256SUMS.txt`.

## Naprawa otwierania PSD

Naprawiono błędne rozpoznawanie dostępnej pamięci w starszym układzie cgroup v1/cgmanager, spotykanym m.in. w MX Linux. Program odmawiał otwarcia PSD wymagającego około 80 MB, zgłaszając limit około 31 MB na komputerze z 32 GB RAM. Wersja 0.17.2 otwierała ten sam plik.

FIX3 rozróżnia nieudostępniony kontroler pamięci od rzeczywistego limitu zero i korzysta z istniejącej, ograniczonej polityki dla nieznanej pamięci. Rzeczywiste, odczytywalne limity pozostają respektowane. Poprawkę potwierdzono na komputerze użytkownika z MX Linux.

- [Opis błędu, diagnoza i sposób odtworzenia](https://github.com/dariuszby-arch/photocraft/blob/v0.20.2-dw3/PSD_MEMORY_REGRESSION_PL.md)
- Dołączony `PSD_memory_regression_18_layers.psd`: publiczny, syntetyczny przykład 1491 × 1055 px, 18 warstw. Generator, opis i licencja znajdują się w [examples/psd-memory](https://github.com/dariuszby-arch/photocraft/tree/v0.20.2-dw3/examples/psd-memory).
- AppImage zawiera diagnostykę: `./PhotoCraft_0.20.2_DW_3_MX23-x86_64.AppImage --memory-report`.

## Zachowane funkcje DW

- JPEG XL: zapis bezstratny i stratny, jakość, effort Fast/Normal, progressive i HDR.
- Zachowanie EXIF/XMP oraz profili ICC w obsługiwanych ścieżkach konwersji JPG/JXL; uzgadnianie znanych znaczników koloru SDR.
- Konwerter wsadowy: kolejka plików i jeden wybrany format dla całej kolejki.

## Weryfikacja i źródła

GitHub Actions buduje oba pakiety osobno na Linuxie i Windowsie. Publikacja wymaga poprawnego numeru wersji, otwarcia przykładowego PSD z 18 warstwami oraz udanej konwersji JPEG XL w obie strony. Linux uruchamia dodatkowo testy wykrywania pamięci i diagnostykę z gotowego AppImage. Testy CLI nie zastępują ręcznej oceny interfejsu na każdym komputerze.

Kod dokładnie tego wydania można pobrać przez automatyczne archiwa **Source code (zip/tar.gz)** pod listą plików. Starsze wydania DW pozostają dostępne.

## English

PhotoCraft **0.20.2-DW3** is an unofficial community build for Linux x86_64 (MX Linux 23 compatible AppImage) and Windows x64 (portable ZIP). It includes PSD FIX3 for legacy cgroup/cgmanager memory detection, while retaining DW JPEG XL metadata, encoding options and batch conversion. Known memory limits remain enforced. The fix was confirmed on the affected MX Linux machine.

Both native GitHub Actions builds must pass version, 18-layer PSD import and JPEG XL encode/decode checks before publication. Downloads include a synthetic PSD example and SHA-256 checksums. Binaries are unsigned. The tagged source archives reproduce the published source revision; this is not an official upstream release.
