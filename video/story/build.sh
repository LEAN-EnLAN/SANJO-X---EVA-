#!/bin/bash
# build.sh <lang>: render every story scene at 1080p30 and join them into one video
set -e; cd "$(dirname "$0")"; LANG_=$1
SC="N01_Ship N02_Earth N03_Collapse N04_Meet N05_Band N06_Helper N07_CPR N08_Sun N09_Shelter N10_Records N11_Mars N12_Close"
for S in $SC; do EVA_LANG=$LANG_ ../../venv-manim/bin/manim -r 1920,1080 --fps 30 --disable_caching -o ${S}_$LANG_ eva_story.py $S >/tmp/b_${S}_$LANG_.log 2>&1 || { echo "$S failed"; exit 1; }; done
D=media/videos/eva_story/1080p30; IN=""; FC=""; i=0
for S in $SC; do IN="$IN -i $D/${S}_$LANG_.mp4"; FC="$FC[$i:v][$i:a]"; i=$((i+1)); done
ffmpeg -v error -y $IN -filter_complex "${FC}concat=n=$i:v=1:a=1[v][a];[a]loudnorm=I=-16:TP=-1.5[an]" -map "[v]" -map "[an]" -c:v libx264 -crf 18 -pix_fmt yuv420p -c:a aac -b:a 192k ../eva_story_$LANG_.mp4
echo done $LANG_
