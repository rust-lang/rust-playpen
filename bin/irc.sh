#!/usr/bin/dash
#
# A shim used to handle queries sent to an IRC bot.

set -o errexit

rustc - -o out <<EOF
#![feature(asm, globs, macro_rules, struct_variant, simd, quad_precision_float)]

extern crate collections;
extern crate native;

#[allow(dead_code)]
static version: &'static str = "$(rustc -v | tail | head -1)";

#[allow(dead_code)]
fn show<T: std::fmt::Show>(e: T) { println!("{}", e) }

fn main() {
    let r = {
        $@
    };
    println!("{}", r)
}
EOF

exec ./out
