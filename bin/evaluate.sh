#!/usr/bin/dash

set -o errexit

rustc -O - -o out <<EOF
$@
EOF

exec ./out
