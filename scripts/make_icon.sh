#!/bin/sh
# Build the OpenASE-Plus icon: rust/assets/icon.png (256 px) and
# rust/assets/icon.ico (16-256 px). Neon cyan eye in a triangle in a
# circle on JASE's dark background, drawn with ImageMagick primitives
# (its SVG renderer drops some strokes). 16 and 24 px use a bolder
# variant without the circle, readable at taskbar size.
# Needs ImageMagick. Usage: scripts/make_icon.sh
set -eu
out=$(cd "$(dirname "$0")/../rust/assets" && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
neon='#19f5e1'
bg='#0b0f14'

# draw <out> <full|small>
draw() {
  if [ "$2" = full ]; then
    convert -size 256x256 xc:none -fill none -stroke "$neon" \
      -strokewidth 9 -draw "circle 128,128 128,30" \
      -draw "stroke-linejoin round polygon 128,30 43,177 213,177" \
      -strokewidth 8 -draw "path 'M 74,128 Q 128,78 182,128 Q 128,176 74,128 Z'" \
      "$tmp/lines.png"
    iris=23; pupil=11
  else
    convert -size 256x256 xc:none -fill none -stroke "$neon" \
      -strokewidth 22 -draw "stroke-linejoin round polygon 128,26 22,212 234,212" \
      -strokewidth 20 -draw "path 'M 58,146 Q 128,84 198,146 Q 128,206 58,146 Z'" \
      "$tmp/lines.png"
    iris=34; pupil=17
  fi
  cy=128; [ "$2" = small ] && cy=146
  convert -size 256x256 xc:none \
    \( -size $((iris * 2))x$((iris * 2)) radial-gradient:'#b6fff7'-'#0a8f86' \) \
    -geometry +$((128 - iris))+$((cy - iris)) -composite \
    \( -size 256x256 xc:none -fill black -draw "circle 128,$cy 128,$((cy - iris))" \) \
    -compose DstIn -composite "$tmp/iris.png"
  convert -size 256x256 xc:none -fill "$bg" -draw "circle 128,$cy 128,$((cy - pupil))" \
    -fill '#e8fffd' -draw "circle $((128 + pupil / 2)),$((cy - pupil / 2)) $((128 + pupil / 2)),$((cy - pupil / 2 - pupil / 3))" \
    "$tmp/pupil.png"
  # Neon glow: a blurred copy of the lines under them.
  convert "$tmp/lines.png" -channel A -blur 0x7 -evaluate multiply 1.6 +channel "$tmp/glow.png"
  convert -size 256x256 xc:none -fill "$bg" -draw "roundrectangle 0,0 255,255 48,48" \
    "$tmp/glow.png" -composite "$tmp/lines.png" -composite \
    "$tmp/iris.png" -composite "$tmp/pupil.png" -composite "PNG32:$1"
}

draw "$out/icon.png" full
draw "$tmp/small.png" small
for side in 16 24; do convert "$tmp/small.png" -resize ${side}x${side} "PNG32:$tmp/$side.png"; done
for side in 32 48 64 128; do convert "$out/icon.png" -resize ${side}x${side} "PNG32:$tmp/$side.png"; done
convert "$tmp/16.png" "$tmp/24.png" "$tmp/32.png" "$tmp/48.png" "$tmp/64.png" "$tmp/128.png" "$out/icon.png" "$out/icon.ico"
