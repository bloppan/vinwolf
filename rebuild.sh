#!/bin/sh

rm /tmp/jam_conformance.sock
cargo build
./target/debug/vinwolf --fuzz

exit 0
