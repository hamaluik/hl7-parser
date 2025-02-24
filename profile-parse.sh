#!/bin/bash
# This script is used to profile the parse function in the profiled-parse binary
# It uses the flamegraph tool to generate a flamegraph of the profiling data
# The flamegraph is then displayed in command line using the flamelens tool
#
# Usage: ./profile-parse.sh
#
# Requirements:
# - cargo-flamegraph
# - flamegraph
# - flamelens
cargo flamegraph --profile profiling --bin profiled-parse --post-process 'flamelens --echo'
