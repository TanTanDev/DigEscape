#!/bin/sh

set -ex

if [ ! -e resources.tar ]; then
  cd ../../resources && tar cf resources.tar * && cd .. && mv resources/resources.tar utils/wasm/
fi

cargo build --target wasm32-unknown-unknown --release
cd ../../
rm -rf static
mkdir static
cp target/wasm32-unknown-unknown/release/dig_escape.wasm static/
cp utils/wasm/index.html static/
cp utils/wasm/gl.js static/
cp utils/wasm/audio.js static/
cp utils/wasm/resources.tar static/
ls -lh static
