#!/bin/sh
# Build rust/assets/flags.png (one 40x30 flag per ISO code, drawn at 20x15,
# 16 per row, in the order of rust/assets/flags.txt) from the SVG flags of
# flag-icons (MIT, https://github.com/lipis/flag-icons, folder flags/4x3).
# Needs ImageMagick and python3. Usage: scripts/make_flag_atlas.sh <dir>
set -eu
src=$1
out=$(cd "$(dirname "$0")/../rust/assets" && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
ls "$src" | grep -E '^[a-z0-9-]+\.svg$' | sort | sed 's/\.svg$//' > "$out/flags.txt"
# ImageMagick's own SVG renderer drops straight stroked lines (the stripes
# of the US flag, some crosses): rewrite them as the rectangles they draw.
python3 -I - "$src" "$tmp" <<'PY'
import pathlib, re, sys
src, tmp = map(pathlib.Path, sys.argv[1:])
path = re.compile(r'<path ([^>]*?)stroke="([^"]+)" stroke-width="([0-9.]+)" d="((?:M[0-9. ]+[hv]-?[0-9.]+)+)"([^>]*)/>')
seg = re.compile(r'M([0-9.]+)[ ]([0-9.]+)([hv])(-?[0-9.]+)')
def rects(m):
    color, width = m.group(2), float(m.group(3))
    out = []
    for x, y, axis, length in seg.findall(m.group(4)):
        x, y, length = float(x), float(y), float(length)
        if axis == "h":
            rx, ry, rw, rh = min(x, x + length), y - width / 2, abs(length), width
        else:
            rx, ry, rw, rh = x - width / 2, min(y, y + length), width, abs(length)
        out.append(f'<rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" fill="{color}"/>')
    return "".join(out)
for svg in src.glob("*.svg"):
    (tmp / svg.name).write_text(path.sub(rects, svg.read_text()))
PY
while read -r code; do
  convert -background none -density 192 "$tmp/$code.svg" -resize '40x30!' "PNG32:$tmp/$code.png"
done < "$out/flags.txt"
cd "$tmp"
montage $(sed 's/$/.png/' "$out/flags.txt") -tile 16x -geometry 40x30+0+0 -background none "PNG32:$out/flags.png"
