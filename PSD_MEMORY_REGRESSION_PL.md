# Regresja importu PSD: nieznany limit cgroups błędnie uznawany za zero RAM

## Krótki opis do przekazania innym

W nowszej kontroli pamięci PhotoCraft wykryto regresję, która na MX Linux
z cgroups v1 i cgmanager może uniemożliwiać otwieranie prawidłowych plików PSD.
Komputer ma 32 GB RAM i około 20 GiB dostępnej pamięci, ale program zgłasza
zbyt niski limit pamięci. Wersja 0.17.2 otwierała pierwotnie zgłoszony plik.

Przyczyną jest błędne rozróżnienie między **nieznanym limitem procesu**
a **rzeczywistym zerem dostępnego RAM**. Proces należy do hierarchii `memory`,
lecz w jego widoku systemu plików zamontowana jest tylko hierarchia `name=systemd`.
Brak możliwości odczytu kontrolera pamięci był zamieniany na `Some(0)`, co
powodowało odmowę importu jeszcze przed dekodowaniem obrazu.

Poprawka FIX3 rozpoznaje nieudostępniony kontroler jako nieznany limit (`None`)
i korzysta z istniejącej polityki zastępczej importu. Odczytywalne limity cgroups
oraz rzeczywiste zero RAM nadal są respektowane. Poprawkę sprawdzono na
odtworzonym układzie kontrolerów, a użytkownik potwierdził działanie na swoim MX.

## Przykładowy plik PSD

[Pobierz PSD_memory_regression_18_layers.psd](examples/psd-memory/PSD_memory_regression_18_layers.psd?raw=true)

To publiczny, syntetyczny przykład: tło i 17 kolorowych prostokątów. Plik nie
zawiera fotografii ani danych z prywatnego dokumentu użytkownika.

| Parametr | Wartość |
|---|---|
| Płótno | 1491 × 1055 pikseli |
| Warstwy | 18 |
| Format | RGB, 8 bitów, kompresja ZIP w PSD |
| Rozmiar pliku | 229 091 bajtów, około 224 KiB |
| Oszacowanie pamięci importu | 189 829 996 bajtów, około 181 MiB |
| Wynik FIX2 przy odtworzonym układzie MX | Odmowa importu: błędny limit 0 bajtów w CLI |
| Wynik FIX3 w tym samym układzie | Poprawny import wszystkich 18 warstw |

Mały rozmiar PSD na dysku wynika z kompresji prostych kształtów. Nie oznacza
małego rozmiaru danych po rozkodowaniu. Plik demonstruje błąd pomiaru pamięci,
a nie uszkodzenie formatu PSD.

Suma SHA-256:

```text
5d95a9a5d5b9a373b312344b51046952718f141f6df181508de6fcd388218ba0
```

[Opis przykładu i odtwarzanie pliku](examples/psd-memory/README.md).

## Pierwotnie zgłoszony objaw

W pierwotnym PSD oszacowanie importu wynosiło **79 598 444 bajty**. Program
odmawiał otwarcia, pokazując limit **31 326 208 bajtów**, mimo około 19,8 GiB
`MemAvailable`. Ten sam plik otwierał się w wersji 0.17.2.

Raport diagnostyczny wersji FIX2:

```text
detector=SystemMemory { available_ram: Some(0), swap_free: Some(0),
  commit_headroom: None, constraint_unreadable: true }
```

Ważne dane systemowe:

```text
MemTotal:       32754536 kB
MemAvailable:   20800996 kB
```

W `/proc/self/cgroup` występował m.in. kontroler `memory`, natomiast spośród
montowań cgroups widoczny był tylko `name=systemd`, z agentem cgmanager.
Oznaczało to nieudostępniony interfejs limitu pamięci, a nie brak fizycznego RAM.

Limit 31 326 208 bajtów pochodził z kredytu na kafelki przechowywane na scratch,
po błędnym wyzerowaniu budżetu RAM. Nie był pomiarem pamięci komputera.
W CLI, bez takiego kredytu scratch, ta sama przyczyna może dać limit 0 bajtów.
Przykład syntetyczny ma inne oszacowanie, ponieważ jego warstwy mają inne dane
i rozmiary niż prywatny PSD; odtwarza tę samą przyczynę odmowy.

## Zmiana w FIX3

- Kompletna lista montowań z hierarchiami v1 bez kontrolera pamięci jest
  rozpoznawana jako `CgroupLimits::Unexposed`.
- Nieznana pojemność procesu jest przekazywana jako `None`, a nie `Some(0)`.
- Importer używa istniejącego progu automatycznego importu do 2 GiB.
  Jest to próg polityki importu, a nie wykryta ilość RAM w komputerze.
- Widoczny kontroler z odczytywalnym limitem nadal ogranicza budżet.
  Uszkodzone dane i nieodczytywalne, widoczne interfejsy zachowują ostrożną obsługę.
- Testy obejmują układ cgmanager, widoczne ograniczenia, błędne wpisy i prawdziwe
  zero pamięci.

Kod: [dostawca pamięci](crates/raster/src/memory/system.rs),
[polityka importu](crates/io/src/memory.rs).

## Weryfikacja

- Dostawca pamięci: 19 testów poprawnych.
- Raster: 84 testy poprawne, 1 pominięty.
- Polityka pamięci importu: 20 testów poprawnych.
- Budżet PSD: 25 testów poprawnych.
- JPEG XL, EXIF/XMP, ICC i 16 bitów: 5 testów poprawnych.
- Clippy pakietów raster i IO, wszystkie cele, bez zależności: poprawny.
- Kontrola warstw: 31 pakietów bez naruszeń.
- WebAssembly: 24 pakiety i dodatkowy wariant HEIF poprawne.
- Gotowy AppImage: pierwotny PSD i przykład otwierają się z 18 warstwami przy
  odtworzonym układzie MX. Konwersja pierwotnego PSD przez bezstratny JXL zachowuje
  identyczne piksele PNG.
- Użytkownik potwierdził poprawne działanie FIX3 na swoim rzeczywistym MX Linux.

Zachowano rozszerzenia DW: JPEG XL, EXIF/XMP i konwerter wsadowy.
[Instrukcja budowania i diagnostyki FIX3](PSD_FIX3_PL.md).
