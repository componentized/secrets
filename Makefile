# This repository builds with its own build, its components and cli exist to smoke test it, see
# README.md

include build/build.mk

# `make test` builds the components before the tests, which otherwise build the composition each
# loads, see crates/test-harness
test: export COMPONENTS_BUILT := true
