#!/usr/bin/dash

set -o errexit

rustc - -C opt-level=$1 --emit=$2 -o ./out
printf '\377' # 255 in octal
exec cat out
