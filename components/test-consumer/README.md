# test-consumer

A consumer of `componentized:secrets`, plugged into each component for its tests, see [`test-harness`](../../crates/test-harness/). Records whether `init` resolves on its first poll.

## Interfaces

Imports:

- `componentized:secrets/store@0.1.1-dev`: the store under test

Exports:

- `componentized:secrets-components/probe`
