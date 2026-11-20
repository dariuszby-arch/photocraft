#!/usr/bin/env bash
# Local Linux build; never writes to GitHub.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
export PHOTOCRAFT_VERSION=0.20.2-DW-PSD-FIX3
export PHOTOCRAFT_OVERRIDE_VERSION="$PHOTOCRAFT_VERSION"
export PHOTOCRAFT_BUILD_SHA="${PHOTOCRAFT_BUILD_SHA:-$(git rev-parse HEAD 2>/dev/null || printf '%s' '9ddbcd336ef67516b5bc78c420e8098e3dacc412')}"
export PHOTOCRAFT_BUILD_DATE="${PHOTOCRAFT_BUILD_DATE:-$(date -u +%F)}"
export GITHUB_REPOSITORY=dariuszby-arch/photocraft
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
source packaging/env.sh
source packaging/linux/verified-download.sh
[[ "$(uname -m)" = x86_64 ]] || { echo 'Ten skrypt buduje AppImage x86_64.' >&2; exit 1; }
skip_build=0
case "${1:-}" in
  '') ;;
  --skip-build) skip_build=1 ;;
  *) echo 'Użycie: bash packaging/linux/buduj_DW_PSD_FIX3.sh [--skip-build]' >&2; exit 2 ;;
esac
fonts_pin=b076417b5880bd4154bd22f00695b1d28c6bea83
mkdir -p "$CARGO_TARGET_DIR/dw-inputs"
fonts_archive="$CARGO_TARGET_DIR/dw-inputs/craft-fonts.tar.gz"
download_verified "https://github.com/storytold/craft-fonts/archive/$fonts_pin.tar.gz" "$fonts_archive" 35eadfeb7e4217a933f24b73a3d190b90fed81e6c933266f092c161746c0a9eb
if [[ ! -f "$CARGO_TARGET_DIR/dw-inputs/craft-fonts-$fonts_pin/fonts/manifest.txt" ]]; then
  tar -xzf "$fonts_archive" -C "$CARGO_TARGET_DIR/dw-inputs"
fi
export CRAFT_FONTS_DIR="$CARGO_TARGET_DIR/dw-inputs/craft-fonts-$fonts_pin"
export CRAFT_FONTS_REQUIRED=1
if [[ "$skip_build" = 0 ]]; then
  cargo build --profile dw-release --locked -p photocraft -p photocraft-cli --features heif --bins
  cargo build --profile dw-release --locked -p photocraft-raster --example system_memory_report
  mkdir -p "$CARGO_TARGET_DIR/release"
  cp "$CARGO_TARGET_DIR/dw-release/photocraft" "$CARGO_TARGET_DIR/dw-release/photocraft-cli" "$CARGO_TARGET_DIR/release/"
fi
# Reuse upstream staging, adding the DW notices before creating the image.
bash packaging/linux/package.sh --skip-build --formats ''
appdir="$CARGO_TARGET_DIR/linux-package/PhotoCraft.AppDir"
cp -a "$CARGO_TARGET_DIR/linux-package/root" "$appdir"
install -m755 packaging/linux/AppRun "$appdir/AppRun"
install -m755 "$CARGO_TARGET_DIR/dw-release/examples/system_memory_report" "$appdir/usr/bin/photocraft-memory-report"
cp packaging/linux/ai.storyteller.photocraft.desktop "$appdir/"
cp assets/app-icon/hicolor/256x256/apps/ai.storyteller.photocraft.png "$appdir/"
ln -s ai.storyteller.photocraft.png "$appdir/.DirIcon"
docs="$appdir/usr/share/doc/photocraft"
cp PSD_FIX3_PL.md NOTICE-JXL.md DW_README_PL.md BATCH_CONVERTER_PL.md JXL_README_PL.md "$docs/"
cp -a third-party/lightcraft "$docs/lightcraft"
tool="$CARGO_TARGET_DIR/appimagetool-1.9.1-x86_64.AppImage"
runtime="$CARGO_TARGET_DIR/type2-runtime-20251108-x86_64"
download_verified https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage "$tool" ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0 true
download_verified https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64 "$runtime" 2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d
output="$DIST/PhotoCraft_0.20.2_DW_PSD_FIX3_MX23-x86_64.AppImage"
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$tool" --no-appstream --runtime-file "$runtime" "$appdir" "$output"
APPIMAGE_EXTRACT_AND_RUN=1 "$output" --cli --version
echo "Gotowe: $output"
