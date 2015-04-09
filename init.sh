#!/bin/bash

set -o errexit -o nounset -o pipefail

cd "${BASH_SOURCE%/*}"

umask 022

rm -rf root-nightly.new
mkdir root-nightly.new

pacstrap -c -d root-nightly.new \
    bash \
    coreutils \
    grep \
    dash \
    filesystem \
    glibc \
    pacman \
    procps-ng \
    shadow \
    util-linux \
    gcc

mkdir root-nightly.new/dev/shm
mknod -m 666 root-nightly.new/dev/null c 1 3
mknod -m 644 root-nightly.new/dev/urandom c 1 9
arch-chroot root-nightly.new useradd -m rust
install -m755 bin/* root-nightly.new/usr/local/bin

rm -rf root-beta.new
cp -a root-nightly.new root-beta.new

curl -O https://static.rust-lang.org/rustup.sh
sh rustup.sh --prefix=root-nightly.new --channel=nightly --components=rustc
sh rustup.sh --prefix=root-beta --channel=beta --components=rustc
rm rustup.sh

[[ -d root-nightly ]] && mv root-nightly root-nightly.old
mv root-nightly.new root-nightly
[[ -d root-nightly.old ]] && rm -rf root-nightly.old

[[ -d root-beta ]] && mv root-beta root-beta.old
mv root-beta.new root-beta
[[ -d root-beta.old ]] && rm -rf root-beta.old
