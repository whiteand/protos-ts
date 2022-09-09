#!/bin/bash

cargo b --release

cp ./target/release/proto-ts ./package/bin/proto-ts-linux
