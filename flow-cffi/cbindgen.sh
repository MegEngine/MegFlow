#!/bin/bash
cbindgen --config cbindgen.toml --crate flow-cffi --output ffi/megflow.h && echo "[v] produce the header file into \"ffi/megflow.h\""
