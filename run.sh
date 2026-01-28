#!/bin/bash
set -e

cargo run --bin generate-tiles
cargo run --bin rouglike-jrpg
