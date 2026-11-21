# PhotoCraft DW 4 — kompozytor GPU na Windows (FXC X3507)

## Objaw

Podczas zmiany krycia, widoczności lub trybu mieszania warstwy program odświeżał cały dokument na CPU. Pojawiał się komunikat o powolnym odświeżeniu (np. 7,2 s) oraz błąd kompilacji potoku `fs_fxinit`:

```text
FXC D3DCompile error
error X3507: 'blend_channel': Not all control paths return a value
```

Komunikat `remembered from an earlier launch` oznacza, że program zapamiętał wcześniejsze niepowodzenie kompilacji tych samych shaderów. Ostrzeżenia `X3571` dotyczące `pow()` pojawiające się obok nie są błędem blokującym kompilację w tym zgłoszeniu.

## Przyczyna i poprawka

Na ścieżce DirectX 12 program używa domyślnie kompilatora FXC dostarczanego z Windows. Przy włączonej optymalizacji FXC odrzucał duży `switch` w `blend_channel`, mimo że każda jego gałąź zwracała wynik. Wystarczyło niepowodzenie jednego potoku, aby kompozytor GPU nie został zbudowany i program przeszedł na CPU.

DW 4 przenosi sprawdzoną poprawkę z projektu głównego: gałęzie przypisują wynik do zmiennej, a funkcja zwraca go jeden raz, po `switch`. Wzory trybów mieszania i obsługa HDR pozostają takie same. Nie dodajemy zewnętrznych DLL ani nie wymuszamy innego sterownika lub interfejsu graficznego.

Źródła poprawki i diagnozy:

- [Upstream issue #8733](https://github.com/storytold/photocraft/issues/8733)
- [Scalona poprawka upstream #8734](https://github.com/storytold/photocraft/pull/8734)
- [Commit scalenia](https://github.com/storytold/photocraft/commit/952be70074793fd0c3f4ef358dbb558bc8842d3e)

## Weryfikacja

Dołączony test `pipelines::every_compositor_pipeline_builds`, przeniesiony z upstream #8734, wymusza DirectX 12 oraz FXC i wyłącza flagi debugowania. To istotne: domyślne flagi testowego urządzenia wyłączały optymalizację FXC i ukrywały problem. Na Windows brak urządzenia DirectX 12 powoduje niepowodzenie testu; runner GitHub Actions korzysta z programowego adaptera WARP.

Publikacja DW 4 wymaga przejścia tego testu oraz testu porównującego tryby mieszania z kompozytorem CPU. Zachowane są również kontrole importu przykładowego PSD z 18 warstwami i konwersji JPEG XL.

Test WARP potwierdza kompilację przez windowsowy FXC i zgodność obliczeń. Działanie oraz czas odświeżenia na konkretnej karcie użytkownika wymagają sprawdzenia na jego komputerze.

## Aktualizacja z DW 3

1. Zamknij PhotoCraft i rozpakuj paczkę DW 4 do nowego katalogu.
2. Uruchom `photocraft.exe` i otwórz dokument.
3. Zmień krycie lub tryb mieszania warstwy. Po zakończeniu początkowej kompilacji sprawdź, czy program używa kompozytora GPU i czy nie powraca `X3507`.

Zmiana treści shadera zmienia jego odcisk w kluczu pamięci nieudanych kompilacji. DW 4 automatycznie podejmuje nową próbę; nie trzeba usuwać ustawień ani pliku pamięci błędów z DW 3. Wybrane wcześniej ręcznie renderowanie CPU nadal jest respektowane.

Pozostają dostępne poprawka pamięci PSD FIX3, opcje JPEG XL, EXIF/XMP i konwerter wsadowy z poprzedniego wydania DW.
