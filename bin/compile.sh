#!/usr/bin/dash

set -o errexit

rustc - -o ./out "$@"
printf '\377' # 255 in octal
exec cat out
