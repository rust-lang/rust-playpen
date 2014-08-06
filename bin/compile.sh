#!/usr/bin/dash

set -o errexit

rustc - --opt-level=$1 --emit=$2 -C llvm-args=-x86-asm-syntax=$3 -o out <<EOF
$4
EOF

printf '\377' # 255 in octal

exec cat out
