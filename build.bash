#!/usr/bin/env sh

set -xe

platform=$1

case $platform in
  linux/amd64)
    RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target=x86_64-unknown-linux-musl
    ls -al ./target
    cp ./target/x86_64-unknown-linux-musl/release/basic-web "./basic-web";;

  linux/arm64)
    RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target=aarch64-unknown-linux-musl
    ls -al ./target
    cp ./target/aarch64-unknown-linux-musl/release/basic-web "./basic-web";;
esac

echo "finished"
