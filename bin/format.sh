#!/usr/bin/dash

set -o errexit

# FIXME #91: use rustfmt.
rustc - -Z unstable-options --pretty -o ./out
printf '\377' # 255 in octal
exec cat out
