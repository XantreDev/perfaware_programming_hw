#!/usr/bin/env nu

def main [...args] {
    echo "use other commands [run]"
    exit 1
}

def "main run" [name: string, ...args] {
    RUSTFLAGS=-Awarnings cargo run --quiet --release --bin $name ...$args
}

def "main run-precise" [name: string, ...args] {
    run-with-features default,precise_rep_test $name ...$args
}

def "main run-with-features-d" [name: string, ...args] {
    run-with-features-d default,precise_rep_test $name ...$args
}


def "main build-with-features-d" [name: string, ...args] {
    build-with-features-d default,precise_rep_test $name ...$args
}

def run-with-features-d [features: string, name: string, ...args] {
    RUSTFLAGS=-Awarnings cargo run --features $features --quiet --profile=release-d --bin $name ...$args
}

def build-with-features-d [features: string, name: string, ...args] {
    RUSTFLAGS=-Awarnings cargo build --features $features --quiet --profile=release-d --bin $name ...$args
}

def run-with-features [features: string, name: string, ...args] {
    RUSTFLAGS=-Awarnings cargo run --features $features --quiet --release --bin $name ...$args
}
def "main run-with-features" [features: string, name: string,  ...args] {
    RUSTFLAGS=-Awarnings cargo run --features $features --quiet --release --bin $name ...$args
}
