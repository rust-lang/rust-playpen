#!/usr/bin/dash

exec rustc - --opt-level=$1 --emit=$2 -o - <<EOF
$3
EOF
