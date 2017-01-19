#!/bin/bash

export OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include
export OPENSSL_LIB_DIR=`brew --prefix openssl`/lib
export DEP_OPENSSL_INCLUDE=`brew --prefix openssl`/include
cargo build "$@"
