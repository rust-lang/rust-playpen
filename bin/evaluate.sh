#!/usr/bin/dash

set -o errexit

bindir=${0%/*}
. $bindir/getopt.sh

TERM=xterm rustc - -o ./out "$@"
printf '\377' # 255 in octal
if [ $coloredtest = 1 ]; then
    # For /evaluate.json, we have {test: true, color: true}. Let's make the
    # output coloured too.
    TERM=xterm exec ./out --color=always
else
    exec ./out
fi
