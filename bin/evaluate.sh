#!/usr/bin/dash

set -o errexit

rustc - --opt-level=$1 -o out <<EOF
$2
EOF

if [ "$3" = "1" ]; then
    printf '\377' # 255 in octal
fi

exec ./out
