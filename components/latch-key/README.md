# latch-key

`componentized:secrets` latch that decides each secret by its key.

The decisions are read from `wasi:config/store`. Each config key is a pattern of secret keys, where `*` matches any characters, including none. Each value is `deferred`, an empty value, or `denied`. A denied `get` fails with `not-found`, hiding whether the secret exists. A key that no pattern matches is deferred, so `*=denied` allows only the keys deferred by another pattern:

```properties
db.*=deferred
db.admin=denied
*=denied

db.password -> DEFERRED (db.*)
db.admin    -> DENIED   (db.admin)
api.token   -> DENIED   (*)
```

When several patterns match a key, the pattern with the most characters that aren't `*` decides, regardless of the order of the config. When the matching patterns are equally specific, `denied` wins.

The config is read once, by the first operation, a change to the config after that is not seen. Every value must be `deferred`, an empty value, or `denied`. Otherwise the cause is logged once at the `critical` level and every operation fails with an `invalid-config<latch-key>` latch error, so a typo is noticed rather than ignored:

```
Invalid config LATCH=latch-key KEY=db.* VALUE=allowed ERROR=expected one of: 'deferred', 'denied'
```

## Interfaces

Imports:

- `wasi:config/store@0.2.0-rc.1`: the decision for each pattern of keys
- `wasi:logging/logging@0.1.0-draft`: logs an invalid config
- `componentized:secrets/store@0.1.2-dev`: the types of the operations being authorized

Exports:

- `componentized:secrets/latch@0.1.2-dev`: the latch
