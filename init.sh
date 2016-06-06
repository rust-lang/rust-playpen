#!/bin/bash

set -o errexit -o nounset -o pipefail

cd "${BASH_SOURCE%/*}"

umask 022

rm -rf root-nightly.new
mkdir root-nightly.new

debootstrap xenial root-nightly.new

chroot root-nightly.new useradd -m rust
chroot root-nightly.new apt-get install -y gcc ca-certificates
install -m755 bin/* root-nightly.new/usr/local/bin

rm -rf root-beta.new
rm -rf root-stable.new
cp -a root-nightly.new root-beta.new
cp -a root-nightly.new root-stable.new

curl -O https://static.rust-lang.org/rustup.sh
for channel in stable beta nightly; do
    sh rustup.sh --prefix=root-${channel}.new --channel=$channel --yes --disable-sudo
    chroot root-$channel.new cargo install -v --root /usr rustfmt
    [[ -d root-$channel ]] && mv root-$channel root-${channel}.old
    mv root-${channel}.new root-$channel
    [[ -d root-${channel}.old ]] && rm -rf root-${channel}.old
done
rm rustup.sh
