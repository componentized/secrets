# from-config

`componentized:secrets`, backed by `wasi:config`, for hosts without secrets, e.g. wasmtime. Each secret is the config value of the same key, a missing key is not found. The config values are read once, when the store is initialized.

> [!WARNING]
> A config value is not a secret, it is available to anything that imports `wasi:config`.

## Interfaces

Imports:

- `wasi:config/store@0.2.0-rc.1`: the values of the secrets

Exports:

- `componentized:secrets/store@0.1.0`
