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
rm -rf root-stable.new
cp -a root-nightly.new root-beta.new
cp -a root-nightly.new root-stable.new

curl -O https://static.rust-lang.org/rustup.sh
for channel in stable beta nightly; do
	sh rustup.sh --prefix=root-${channel}.new --channel=$channel --yes
	[[ -d root-$channel ]] && mv root-$channel root-${channel}.old
	mv root-${channel}.new root-$channel
	[[ -d root-${channel}.old ]] && rm -rf root-${channel}.old
done
rm rustup.sh
