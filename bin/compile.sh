#!/usr/bin/dash

echo "$3" | exec rustc - --opt-level=$1 --emit=$2 -o -
