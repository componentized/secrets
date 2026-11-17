# test-consumer

A consumer of `componentized:secrets`, plugged into each component for its tests, see [`tests`](../../tests/). Records whether `init` resolves on its first poll.

## Interfaces

Imports:

- `componentized:secrets/store@0.1.0`: the store under test

Exports:

- `componentized:secrets-components/probe`
