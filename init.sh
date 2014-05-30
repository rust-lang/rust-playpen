#!/bin/bash

cd "${BASH_SOURCE%/*}" || exit

umask 022

mkdir root-master.new

pacstrap -c -d root-master.new \
    bash \
    coreutils \
    grep \
    dash \
    filesystem \
    glibc \
    pacman \
    procps-ng \
    shadow \
    util-linux

mkdir root-master.new/dev/shm
mknod -m 644 root-master.new/dev/urandom c 1 9
arch-chroot root-master.new useradd -m -g users -s /bin/bash rust
install -m755 bin/* root-master.new/usr/local/bin

cp -a root-master.new root-0.10.new
pacman -r root-master.new -S rust-git --noconfirm
pacman -r root-0.10.new -S rust --noconfirm

[[ -d root-master ]] && mv root-master root-master.old
mv root-master.new root-master
[[ -d root-master.old ]] && rm -rf root-master.old

[[ -d root-0.10 ]] && mv root-0.10 root-0.10.old
mv root-0.10.new root-0.10
[[ -d root-0.10.old ]] && rm -rf root-0.10.old
