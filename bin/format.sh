#!/usr/bin/dash

set -o errexit

rustc - --pretty -o out <<EOF
$1
EOF

printf '\377' # 255 in octal

exec cat out
