#!/usr/bin/env nu

def main [...args] {
    echo "use other commands [run]"
    exit 1
}

def "main run" [name: string, ...args] {
    RUSTFLAGS=-Awarnings cargo run --quiet --release --bin $name ...$args
}
