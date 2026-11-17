# PhotoCraft 0.20.2 DW — konwersja wielu zdjęć

Otwórz **File → Export → Batch Converter…**. Nie trzeba wcześniej otwierać zdjęcia w edytorze.

1. Wybierz **Dodaj plik…**, **Dodaj pliki…** albo **Dodaj katalog…**.
2. Dla katalogu ustaw filtr, np. `*.jpg;*.png;*.jxl`, i opcjonalnie uwzględnij podkatalogi.
   Filtr dotyczy dodawania katalogu; wielkość liter nie ma znaczenia.
3. Checkboxy przy plikach określają, które zdjęcia zostaną przekonwertowane.
   Przyciski **Wszystkie** i **Żadne** zmieniają zaznaczenie całej kolejki.
4. Pod listą wybierz **Katalog wynikowy**.
5. Po prawej wybierz jeden format i ustawienia dla całej kolejki, następnie naciśnij
   **Rozpocznij konwersję**.

Lista pokazuje stany **Oczekuje**, **W toku**, **Gotowe**, **Pominięto**, **Błąd** i **Przerwano**.
Najedź na nazwę zdjęcia, żeby zobaczyć pełną ścieżkę źródłową, wynikową i uwagi.
Pod listą znajduje się postęp z liczbą plików i podsumowanie wykonania.
Uszkodzony plik nie zatrzymuje pozostałych zdjęć. **Zatrzymaj** przerywa pracę;
gotowe wyniki pozostają na dysku. Zamknięcie okna konwertera także zgłasza zatrzymanie.

## JPEG XL i metadane

Domyślnie wybrane są JPEG XL, jakość 85 oraz **Metadata: All**.
Wyłączone **Lossless** pozwala ustawiać jakość 1–100; włączone oznacza kompresję bezstratną.
Opcjonalny limit KB jest sprawdzany na faktycznie zakodowanym pliku.
**Metadata: All** i wyłączone **Remove location info** zachowują dostępne EXIF i XMP
przy konwersji JPG ↔ JXL. Żadne metadane nie zostaną odtworzone, jeśli źródło ich nie zawiera.
AVIF, EXR, HDR, TGA i ICO mają ograniczenia zapisu metadanych opisane w panelu ustawień.

Konwerter korzysta z tego samego skalowania, wyostrzania, znaku wodnego i eksportu co
**Export As**. Rozmiar jest wyliczany oddzielnie dla każdego źródła, z jego proporcji.
Źródła i dokumenty otwarte w edytorze pozostają bez zmian.

Przy jednakowych nazwach domyślne **Add number** dopisuje numer.
**Overwrite** nie może zastąpić żadnego źródła z kolejki ani wcześniejszego wyniku tej samej kolejki.
**Skip** pomija istniejący plik. Wynik jest zapisywany atomowo po ukończeniu kodowania.
Kolejka mieści do 10 000 plików; ponowne dodanie tego samego pliku nie tworzy duplikatu.

## Uruchomienie AppImage na MX Linux

Nadaj plikowi prawo uruchamiania we właściwościach i uruchom go dwuklikiem.
W razie braku FUSE można użyć `APPIMAGE_EXTRACT_AND_RUN=1` przed poleceniem uruchomienia.
Kodeki JPEG XL są wbudowane — `cjxl` i `djxl` nie są wymagane.
