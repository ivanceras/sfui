#!/bin/bash
set -ev

wasm-pack build --target=web

basic-http-server ./ -a 0.0.0.0:3337
