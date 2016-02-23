#!/usr/bin/dash

set -o errexit

RUST_BACKTRACE=1 TERM=xterm rustc - -o ./out "$@"
printf '\377' # 255 in octal
exec cat out
