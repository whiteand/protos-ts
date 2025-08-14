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