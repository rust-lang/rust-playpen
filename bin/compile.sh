#!/usr/bin/dash

set -o errexit

bindir=${0%/*}
. $bindir/getopt.sh

TERM=xterm rustc - -o ./out "$@"
printf '\377' # 255 in octal
exec cat out
