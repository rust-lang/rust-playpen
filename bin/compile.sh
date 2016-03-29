#!/usr/bin/dash

set -o errexit

if [ "$1" = "--backtrace" ]; then
    export RUST_BACKTRACE=1
    shift
fi

TERM=xterm rustc - -o ./out "$@"
printf '\377' # 255 in octal
exec cat out
