#!/bin/sh

export RUSTFLAGS="-C link-arg=-Wl,-undefined,dynamic_lookup"
exec cargo make verify
