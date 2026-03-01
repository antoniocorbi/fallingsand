
#set shell := ["fish", "-c"]

# List this help
@default:
    just --list

# Update packages
@update:
    cargo update

# Where are we?
system-info:
  @echo "This is an {{arch()}} machine".

# Build frtree App
alias b := build
@build:
    echo Building frtree app
    cargo b

# Build web app
alias bw := bweb
@bweb:
    echo Building frtree web app
    #trunk --config Trunk-upload.toml -v build
    trunk build

alias rw := rweb
# Run web app
@rweb:
    echo Serving formulars WEB app...
    trunk serve

alias r := run
# Run formulars App
run:
    cargo r --release

# Count lines of code
@sloc:
    tokei src/

# List files in src
@lf:
    #! /bin/sh
    for f in src/*.rs
    do
      ls -l $f
    done

@ff:
    find {{invocation_directory()}}/src -name \*.rs -exec ls -l {} \;

# Format source files
@format:
    find {{invocation_directory()}}/src -name \*.rs -exec rustfmt {} \;

# Clean leftovers
@clean:
    cargo clean
    find . -name "*~" -delete

alias c := check
# Standard checks
@check:
    cargo check

# Clippy checks
@cl:
    cargo clippy -- -D warnings -W clippy::pedantic

## Docs  ------------------------------------------------------------------

# Generate docs
@doc:
    cargo doc --no-deps --document-private-items

## Tests ------------------------------------------------------------------

# test everything
alias ta := test-all
test-all:
    cargo nextest run

# run a specific test
test TEST:
    ./test --test {{TEST}}
