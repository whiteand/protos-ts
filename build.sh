#!/bin/bash

cargo b --release

mkdir ./package/bin
cp -r ./target/release/protos-ts ./package/bin/protos-ts-linux
