#!/usr/bin/dash

set -o errexit

rustc - -Z unstable-options --pretty -o ./out
printf '\377' # 255 in octal
exec cat out
