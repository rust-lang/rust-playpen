#!/usr/bin/dash

exec rustc - --emit=$1 -o - <<EOF
$2
EOF
