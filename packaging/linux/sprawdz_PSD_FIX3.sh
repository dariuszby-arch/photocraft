#!/usr/bin/env bash
# Diagnostyka tylko do odczytu. Nie modyfikuje PSD ani ustawień programu.
set -uo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
appimage="$script_dir/PhotoCraft_0.20.2_DW_PSD_FIX3_MX23-x86_64.AppImage"
if [[ ! -f "$appimage" ]]; then
  echo 'Umieść skrypt w tym samym katalogu co AppImage PSD FIX3.' >&2
  exit 2
fi
chmod +x "$appimage" || exit 2
APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" --cli --version
APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" --memory-report
if [[ $# -gt 0 ]]; then
  echo
  echo 'Próba odczytu wskazanego PSD bez zapisywania zmian:'
  APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" --cli info --compact "$1"
fi
