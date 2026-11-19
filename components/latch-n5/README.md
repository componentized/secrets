# latch-n5

`componentized:secrets` latch that aggregates 5 latches, any latch can deny an operation.

The latches are asked in order, `latch0`, `latch1`, `latch2`, `latch3`, `latch4`, and the first denial is the decision. Latches after a denial are not asked, but every latch observes the final decision with `observe-decision`, even when an earlier latch fails to observe it. The first failure is returned. A latch error from `authorize` is returned without asking the latches after it.

## Interfaces

Imports:

- `latch0`: `componentized:secrets/latch@0.1.1`, a latch to aggregate
- `latch1`: `componentized:secrets/latch@0.1.1`, a latch to aggregate
- `latch2`: `componentized:secrets/latch@0.1.1`, a latch to aggregate
- `latch3`: `componentized:secrets/latch@0.1.1`, a latch to aggregate
- `latch4`: `componentized:secrets/latch@0.1.1`, a latch to aggregate
- `componentized:secrets/store@0.1.1`: the types of the operations being authorized

Exports:

- `componentized:secrets/latch@0.1.1`: the latch
