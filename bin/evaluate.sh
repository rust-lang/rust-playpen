#!/usr/bin/dash

set -o errexit

echo "$2" | rustc - --opt-level=$1 -o out

exec ./out
