# How to build

## Install cross

```sh
cargo install cross --git https://github.com/cross-rs/cross
```

## Run build

```sh
./build.sh
```

## Increase version of the package

```sh
cd ./package && npm version patch
```

## Publish package 

```sh
cd ./package && npm publish
```

# How to publish

## Build macos binary


```sh
cross build --release --target aarch64-apple-darwin && cp -r ./target/aarch64-apple-darwin/release/protos-ts ./package/bin/protos-ts-macos
```

## Increment version

## Add tag starting with v

## Push
