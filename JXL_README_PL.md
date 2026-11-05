# PhotoCraft 0.20.2 DW z obsługą JPEG XL

Mechanizm JXL przeniesiony z dostarczonych źródeł LightCraft.

## Korzystanie

1. Nadaj plikowi AppImage prawo uruchamiania w jego właściwościach lub poleceniem:
   `chmod +x PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage`
2. Uruchom AppImage dwukrotnym kliknięciem.
3. Otwieranie: File → Open, filtr JPEG XL; działa również przeciąganie `.jxl` do programu.
4. Zapis bezstratny: File → Save As, format JPEG XL (`.jxl`).
5. Eksport z ustawieniami: File → Export → Export As, format JPEG XL.
   Pole Lossless włącza zapis bezstratny. Po jego wyłączeniu suwak Quality ustala jakość 1–100.
   Jakość 90 jest ustawieniem początkowym kompresji stratnej.

Kodek jest wbudowany: nie trzeba instalować `cjxl`, `djxl` ani systemowej biblioteki libjxl.
AppImage dotyczy komputerów Linux x86_64. W razie problemu z FUSE można uruchomić poleceniem
`APPIMAGE_EXTRACT_AND_RUN=1 ./PhotoCraft_0.20.2_DW_1_MX23-x86_64.AppImage`.
Program potrzebuje działającego środowiska
graficznego i sterownika obsługiwanego przez wgpu/Vulkan lub OpenGL.

## Obsługiwane dane

- Odczyt JXL z kontenerem lub bez kontenera, identyfikacja z sygnatury pliku.
- Zapis bezstratny i stratny RGB/RGBA: 8 albo 16 bitów na kanał.
- Przezroczystość jest zachowywana; szarości przy zapisie są rozszerzane do RGB.
- ICC w eksporcie bezstratnym; EXIF i XMP według wyboru zachowania metadanych.
- Przy eksporcie stratnym dokument jest konwertowany przez swój profil ICC do sRGB.
- PhotoCraft ostrzega o spłaszczeniu warstw i ograniczeniach docelowego formatu.
  JXL nie zachowuje edytowalnych warstw; do tego służą PSD lub `.pcraft`.
- Eksport obrazów zmiennoprzecinkowych korzysta z istniejącej konwersji PhotoCraft do
  próbek całkowitych i zgłasza redukcję precyzji. Nie jest to bezstratny eksport HDR 32-bit.
- Animowane JXL otwierają się jako pierwsza klatka z ostrzeżeniem.
- Odczyt ma limity wymiarów i pamięci oraz obsługę błędnych i uciętych plików.
- Opcje wysiłku kodera i zapisu progresywnego są dostępne w Rust API (`EncodeOptions.jxl`).

## Samodzielna kompilacja na MX Linux 23 / Debianie

Potrzebny Rust 1.95 lub nowszy (`cargo --version`). Przygotowanie narzędzi:

```sh
sudo apt update
sudo apt install build-essential pkg-config cmake libxkbcommon-dev libwayland-dev \
  libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev squashfs-tools \
  desktop-file-utils patchelf curl
bash Zbuduj_AppImage_MX_Linux.sh
```

Skrypt używa dwóch zadań kompilacji, aby ograniczyć zużycie RAM. Pierwsza kompilacja
pobiera zależności z Internetu. Wynik znajduje się w `dist/release/`.
Wariant JXL nie zawiera metadanych aktualizacji AppImage do oficjalnego wydania.
Oryginalne licencje i informacje o pochodzeniu kodu są w `NOTICE-JXL.md` i
`third-party/lightcraft/`.

## Weryfikacja

Wyniki testów oraz szczegóły gotowej paczki znajdują się w `RAPORT_TESTOW_DW.md`.

Rozbudowany panel eksportu opisano w `EXPORT_README_PL.md`.
