#!/bin/bash

cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
cross build --release --target aarch64-apple-darwin

rm -rf ./package/bin
mkdir ./package/bin
cp -r ./target/x86_64-pc-windows-gnu/release/protos-ts.exe ./package/bin/protos-ts-win.exe
cp -r ./target/aarch64-apple-darwin/release/protos-ts ./package/bin/protos-ts-macos
cp -r ./target/x86_64-unknown-linux-gnu/release/protos-ts ./package/bin/protos-ts-linux