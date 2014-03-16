#!/usr/bin/dash
#
# A shim used to handle queries sent to an IRC bot.

set -o errexit

rustc - -o out <<EOF
#[feature(globs, macro_rules, struct_variant, simd, asm)];

extern crate collections;
extern crate native;

#[allow(dead_code)]
static version: &'static str = "$(rustc -v | tail | head -1)";

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let r = {
        $@
    };
    println!("{}", r)
}
EOF

exec ./out
