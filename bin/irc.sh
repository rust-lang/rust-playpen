#!/usr/bin/dash
#
# A shim used to handle queries sent to an IRC bot.

set -o errexit

rustc - -o out <<EOF
#![feature(asm, globs, macro_rules, phase, simd, struct_variant, thread_local, quad_precision_float)]
#![allow(dead_code)]

extern crate collections;
extern crate native;
extern crate rand;
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

static version: &'static str = "$(rustc -v | tail | head -1)";

fn show<T: std::fmt::Show>(e: T) { println!("{}", e) }

fn main() {
    let r = {
        $1
    };
    println!("{}", r)
}
EOF

exec ./out
