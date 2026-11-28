# PhotoCraft DW — integracja JPEG XL, 6 listopada 2026

Przygotowano lokalną gałąź `fix/dw-upstream-20261106` na podstawie oficjalnego
`main` o SHA `a259993118ac7ce1d4c828a385abbc3191841e86`.
Źródło funkcji DW: `b93ffc7346a2c850b1039c742f99b835dcc8e5ff` oraz końcowa
gałąź DW `451fd0b5f3c21e243670f3ba67a76b3abe9a2120`.
Poprawki EXIF/XMP zaczerpnięto z PR #11556, head
`a50d4e042a78aa6c418578aaf2c52e6f7ca2abd3`.

## Zmiany

- Przeniesiono funkcjonalny kod DW na aktualny main, rozwiązując konflikty CLI,
  aktualizacji klatki interfejsu i okna eksportu. Zachowano kontrolę pamięci
  przed zapisem oraz aktualne zachowanie okien upstreamu.
- Zachowano odczyt JXL, lossless/lossy, jakość, progressive, HDR, ICC,
  dwa rzeczywiście obsługiwane tryby effort (Fast/Normal) i konwerter seryjny.
- Połączono zachowywanie EXIF/XMP z korektą oznaczeń sRGB. Dla lossless
  z osadzonym profilem źródłowym nie zmienia się oznaczeń na sRGB.
- Podstawowy kodek także zapisuje opcjonalne EXIF/XMP; ustawienia, których
  nie obsługuje, zwracają błąd zamiast być ignorowane.
- Poprawiono konwersję P3 → sRGB przy wyłączonym osadzaniu ICC w lossless.
  Przygotowanie kolorów jest wspólne dla eksportu przez CPU i własny renderer/GPU.
- Rozdzielono parser korekt EXIF na mniejsze funkcje, aby spełniał limit
  złożoności Clippy bez wyłączania kontroli.
- Uaktualniono dokumentację możliwości kodeka i informacje licencyjne.

## Zastosowanie

Pełne źródła znajdują się w katalogu `PhotoCraft_DW/` w paczce.
`DW_na_main.patch` stosuje się do wskazanego powyżej oficjalnego main,
nie do starej pełnej gałęzi DW. `PR11556_clippy.patch` jest osobną małą
poprawką parsera EXIF dla istniejącej gałęzi PR-a metadanych.
Są to alternatywne ścieżki użycia; nie należy nakładać obu patchy na tę samą gałąź.

Nie zmieniano repozytoriów GitHub, PR-ów, wydań ani workflow publikacji.
Paczka zawiera kod źródłowy i poprawki; nie jest instalatorem EXE ani AppImage.
Numer wersji pozostawiono zgodny z oficjalnym main (manifest: 0.20.1).

## Weryfikacja

Środowisko: Linux x86_64, Rust 1.99.1. Testy wykorzystują obrazy syntetyczne.

| Sprawdzenie | Wynik |
|---|---|
| Kodeki, domyślne funkcje: `--lib` + `jxl_export` | 165 + 6 testów OK; 1 test pominięty zgodnie z oznaczeniem upstream |
| Integracja kodeka JXL, `--test it jxl` | 5 testów OK |
| Kodeki bez domyślnych funkcji: `--lib` + `jxl_export` | 166 + 6 testów OK; 1 test pominięty zgodnie z oznaczeniem upstream |
| IO, `--test it jxl` | 5 testów OK: metadane w obu kierunkach, ICC/16 bit, P3 bez ICC, trailing boxes |
| Engine, `--test export_panel` | 9 testów OK: HDR, rozdzielczość, watermark, limit pliku, redakcja metadanych |
| UI, `--lib export_dialog` | 49 testów OK; 3 specjalne testy pominięte domyślnie |
| Rzeczywista kolejka z menu | 1 test OK, w tym błędny plik, niezaznaczony plik i JPG/JXL/JPG |
| Renderowanie okien poza ekranem | 1 test OK; obejrzano oba obrazy |
| Test profilu 24 MP, uruchomiony osobno | OK: 5,58236 ms; wszystkie kafle pikseli współdzielone |
| `panic_hunt -- --ignored` | 1 test OK: polecenia nie panikują ani nie zawieszają się na błędnych parametrach |
| Clippy, codecs/io/engine/ui-egui, wszystkie cele, `-D warnings` | OK |
| `cargo fmt --all -- --check`, `git diff --check` | OK |
| `cargo xtask layers` | 31 modułów, bez naruszeń warstw |
| `cargo xtask wasm` | 24 biblioteki + kodeki z `heif`: OK |
| `cargo xtask parity` | 660/660 pozycji menu aktywnych |

Szersze testy wykryły i pozwoliły naprawić regresję DW: niepotrzebną konwersję
przy eksporcie do już zgodnego profilu RGB. Eksport zachowuje teraz kafle obrazu.

Nie jest to deklaracja wykonania całego zestawu testów projektu.
Próba `cargo xtask test-corpus -p codecs -p io --local -- jxl` zatrzymała się
na braku lokalnego repozytorium `photocraft-corpus`; zewnętrznego pełnego
corpus nie zweryfikowano. Nie przygotowano ani nie testowano binariów Windows.
Wasm zachowuje ostrzeżenia o nieużywanych funkcjach w istniejącym kodzie upstreamu;
kontrola zakończyła się powodzeniem.

## Źródła porównania

- [Oficjalny main](https://github.com/storytold/photocraft/commit/a259993118ac7ce1d4c828a385abbc3191841e86)
- [Port DW](https://github.com/dariuszby-arch/photocraft/commit/b93ffc7346a2c850b1039c742f99b835dcc8e5ff)
- [PR #11556 — EXIF/XMP](https://github.com/storytold/photocraft/pull/11556)
