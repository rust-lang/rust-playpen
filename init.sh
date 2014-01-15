#!/bin/bash

umask 022

rm -rf root-master
mkdir root-master

pacstrap -c -d root-master \
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

mknod -m 644 root-master/dev/urandom c 1 9
mknod -m 666 root-master/dev/null c 1 3
arch-chroot root-master useradd -m -g users -s /bin/bash rust
install -m755 bin/* root-master/usr/local/bin

pacman -r root-master -S rust-git --noconfirm
