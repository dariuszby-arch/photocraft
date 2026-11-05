# PhotoCraft 0.20.2 DW — EXIF i XMP

Przeniesiono poprawkę odczytu metadanych umieszczonych również za ostatnią
klatką kontenera JXL. Eksport korzysta z bieżącego mechanizmu PhotoCraft 0.20.2.

Przy konwersji JPG → JXL → JPG wybierz **Metadata: All**. Jeśli chcesz zachować
również lokalizację, wyłącz **Remove location info**. Opcje redakcji metadanych
działają celowo; program nie odtwarza danych, których nie było w źródle.

Przeniesione testy obejmują oba kierunki konwersji, tryb stratny i bezstratny,
metadane za obrazem i wyłączenie eksportu metadanych. Wykonanie i wyniki dla
konkretnej kompilacji opisuje raport dołączony do paczki.
