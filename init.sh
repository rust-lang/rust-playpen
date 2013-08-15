#!/bin/bash

umask 022

rm -rf root-master root-0.7
mkdir root-master

pacstrap -c -d root-master \
    bash \
    coreutils \
    dash \
    filesystem \
    gcc-libs \
    glibc \
    grep \
    pacman \
    procps-ng \
    shadow \
    util-linux

mknod root-master/dev/urandom c 1 9
arch-chroot root-master useradd -m -g users -s /bin/bash rust
install -m755 bin/* root-master/usr/local/bin

cp -a root-master root-0.7
pacman -r root-master -S rust-git --noconfirm
pacman -r root-0.7 -S rust --noconfirm
