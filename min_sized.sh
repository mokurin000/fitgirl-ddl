#!/usr/bin/env bash

RUSTFLAGS='-C link-arg=-Wl,-z,pack-relative-relocs' \
    cargo build --release -Z build-std=std,panic_abort

