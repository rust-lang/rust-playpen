#!/bin/dash

set -o errexit

if [ "$RUST_PLAYPEN_ENV" != "irc" ]; then
    rustc --version
fi

RUST_NEW_ERROR_FORMAT=1 TERM=xterm rustc - -o ./out "$@"
printf '\377' # 255 in octal
exec cat out
