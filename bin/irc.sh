#!/usr/bin/dash
#
# A shim used to handle queries sent to an IRC bot.

set -o errexit

rustc - -o out <<EOF
#[feature(globs, macro_rules, struct_variant)];

extern mod extra;

static version: &'static str = "$(rustc -v | head -1)";

fn main() {
    let r = {
        $@
    };
    println!("{:?}", r)
}
EOF

exec ./out
