#!/bin/sh
# Build rust/assets/flags.png (one 20x15 flag per ISO code, 16 per row, in
# the order of rust/assets/flags.txt) from the SVG flags of flag-icons
# (MIT, https://github.com/lipis/flag-icons, folder flags/4x3).
# Needs ImageMagick. Usage: scripts/make_flag_atlas.sh <flag-icons>/flags/4x3
set -eu
src=$1
out=$(cd "$(dirname "$0")/../rust/assets" && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
ls "$src" | grep -E '^[a-z0-9-]+\.svg$' | sort | sed 's/\.svg$//' > "$out/flags.txt"
while read -r code; do
  convert -background none -density 192 "$src/$code.svg" -resize '20x15!' "PNG32:$tmp/$code.png"
done < "$out/flags.txt"
cd "$tmp"
montage $(sed 's/$/.png/' "$out/flags.txt") -tile 16x -geometry 20x15+0+0 -background none "PNG32:$out/flags.png"
