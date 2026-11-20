# Syntetyczny PSD do odtworzenia regresji kontroli pamięci

Plik: [PSD_memory_regression_18_layers.psd](PSD_memory_regression_18_layers.psd?raw=true).

1491 × 1055 pikseli, RGB 8-bit, 18 warstw: jasne tło i 17 kolorowych prostokątów.
Cała grafika powstaje programowo; nie używa fotografii, czcionek ani zewnętrznych
zasobów. Licencja: MIT OR Apache-2.0, zobacz [LICENSE.txt](LICENSE.txt).

Plik ma 229 091 bajtów. Oszacowanie importu wynosi 189 829 996 bajtów; warstwy
przechowują pełne prostokąty płótna, choć widoczna grafika jest prosta.

## Odtworzenie

```bash
cargo run --locked --profile dw-release -p photocraft-psd \
  --example memory_admission_repro -- /tmp/PSD_memory_regression_18_layers.psd
```

Generator wymaga nowej ścieżki wyjściowej; istniejący plik pozostaje chroniony.

SHA-256 dołączonego PSD:

```text
5d95a9a5d5b9a373b312344b51046952718f141f6df181508de6fcd388218ba0
```

## Co sprawdzono

W odtworzonym układzie MX/cgmanager FIX2 odrzuca ten plik przed dekodowaniem
z błędnym limitem 0 bajtów w CLI. FIX3 importuje wszystkie 18 warstw poprawnie.
W zwykłym środowisku z prawidłowo odczytywanym budżetem RAM błąd może nie wystąpić;
nie jest on własnością samego pliku.

[Opis regresji i warunków występowania](../../PSD_MEMORY_REGRESSION_PL.md).
