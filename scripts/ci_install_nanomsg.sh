#!/usr/bin/env bash
# -*- coding: utf-8 -*-

VERSION="1.2"
BUILD_DIR="nanomsg-${VERSION}/build"

pushd /tmp || exit 1

sudo apt install build-essential wget cmake -y \
    && wget -O "nanomsg.zip" "https://github.com/nanomsg/nanomsg/archive/refs/tags/${VERSION}.zip" \
    && unzip "nanomsg.zip" \
    || exit 2

mkdir -p $BUILD_DIR && cd $BUILD_DIR || exit 1

cmake .. \
    && cmake --build . \
    && sudo cmake --build . --target install \
    && sudo ldconfig \
    || exit 3

popd || exit 1
