#!/bin/bash
set -ev

wasm-pack build --release --target=web

basic-http-server ./ -a 0.0.0.0:3338
