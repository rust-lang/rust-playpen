#!/usr/bin/dash

echo "$1" | exec rustc - --pretty
