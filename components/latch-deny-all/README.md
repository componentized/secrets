# latch-deny-all

`componentized:secrets` latch that denies every operation with `not-found`, allowing no secrets.

## Interfaces

Imports:

- `componentized:secrets/store@0.1.2-dev`: the types of the operations being authorized

Exports:

- `componentized:secrets/latch@0.1.2-dev`: the latch
