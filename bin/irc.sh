#!/usr/bin/dash
#
# A shim used to handle queries sent to an IRC bot.

set -o errexit

rustc - -o out <<EOF
#[feature(globs, macro_rules, struct_variant)];

extern crate extra;

#[allow(dead_code)]
static version: &'static str = "$(rustc -v | tail | head -1)";

fn main() {
    let r = {
        $@
    };
    println!("{:?}", r)
}
EOF

exec ./out
