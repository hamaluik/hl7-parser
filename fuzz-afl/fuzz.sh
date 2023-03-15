#!/bin/bash
cargo afl fuzz -d -i ../test_assets -o out target/debug/fuzz-afl
