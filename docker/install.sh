#!/bin/sh

curl https://static.rust-lang.org/rustup.sh | \
  sh -s -- --disable-sudo --channel=$1 -y
cargo install --debug rustfmt --root /usr/local
