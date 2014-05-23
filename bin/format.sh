#!/usr/bin/dash

exec rustc - --pretty <<EOF
$1
EOF
