#!/bin/sh

set -ex

cd ../../resources && tar cf resources.tar * && cd .. && mv resources/resources.tar utils/wasm/
cp utils/wasm/resources.tar src/
ls -lh static
