#!/usr/bin/dash

exec rustc - --pretty <<EOF
$@
EOF
