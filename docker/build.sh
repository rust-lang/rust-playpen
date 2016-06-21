#!/bin/sh

for channel in stable beta nightly; do
    docker build \
        --no-cache \
        --force-rm \
        --pull \
        --rm \
        --tag rust-playpen-$channel \
        --file docker/Dockerfile-$channel \
        docker
done
