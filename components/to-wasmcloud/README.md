# to-wasmcloud

Adapts a `componentized:secrets` store to `wasmcloud:secrets`, for components that use the secrets of a wasmCloud host. Plug a store into it, e.g. [`from-config`](../from-config/).

`wasmcloud:secrets` has no `init`. The first `get` initializes the store. If that fails, the next `get` tries again. A secret holds its value, and the store's secrets don't change once it is initialized, so `reveal` returns the value `get` found.

Errors map to `wasmcloud:secrets` as follows:

| `componentized:secrets` | `wasmcloud:secrets` |
| --- | --- |
| `not-found` | `not-found` |
| `other(some(message))` | `upstream(message)` |
| `other(none)` | `upstream("other")` |
| `not-initialized` | `upstream("not-initialized")` |

`wasmcloud:secrets/secret` is not exported. It names its secret by the label it is imported with, which the adapter can't know.

## Interfaces

Imports:

- `componentized:secrets/store@0.1.2-dev`: the store of the secrets

Exports:

- `wasmcloud:secrets/store@2.1.0`
- `wasmcloud:secrets/reveal@2.1.0`
