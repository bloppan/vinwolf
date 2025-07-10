#!/bin/sh

rm /tmp/jam_conformance.sock
rm /tmp/jam_target.sock
cargo build
./target/debug/vinwolf --fuzz

exit 0
