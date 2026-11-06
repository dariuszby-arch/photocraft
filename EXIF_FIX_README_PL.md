# PhotoCraft — zachowanie EXIF i XMP

Poprawiono odczyt EXIF/XMP znajdujących się za ostatnią klatką kontenera JXL.
Regresje obejmują ten przypadek oraz konwersje JPG → JXL → JPG w trybie
bezstratnym i stratnym. Panel eksportu sprawdza ustawienie Metadata: All
przy zapisie JPG i JXL.

Dla zachowania całych metadanych wybierz File → Export → Export As,
Metadata: All i wyłącz Remove location info. Opcja Remove location info
usuwa dane lokalizacji także wtedy, gdy wybrano All.

Bieżący raport wersji 0.17.2: RAPORT_TESTOW_DW.md.
RAPORT_TESTOW_DW.md zawiera archiwalny raport poprawki 0.17.1.
