#!/bin/bash

cargo b --release
cargo b --release --target x86_64-pc-windows-gnu

rm -rf ./package/bin
mkdir ./package/bin
cp -r ./target/release/protos-ts ./package/bin/protos-ts-linux
cp -r ./target/x86_64-pc-windows-gnu/release/protos-ts.exe ./package/bin/protos-ts-win.exe