# gate

`componentized:secrets` gate, access control for the secrets of a store.

The latch is asked to authorize each `get`, then told the final decision with `observe-decision`. A deferred `get` gets the secret from the store. A denied `get` fails with the latch's reason, without calling the store, and is logged as a warning. The key is logged, the secret never is:

```
Denied REASON=not-found OPERATION=componentized:secrets/store#get KEY=db.admin
```

A latch error, or a failure to observe the decision, fails the `get` with `other`, prefixed with `latch-error:`, and is logged as an error:

```
Latch error CODE=invalid-config<latch-key> OPERATION=componentized:secrets/store#get KEY=db.admin
```

`init` is not authorized, it gets no secret, and initializes the store.

## Interfaces

Imports:

- `componentized:secrets/latch@0.1.1`: decides whether each `get` may proceed
- `wasi:logging/logging@0.1.0-draft`: logs denials and latch errors
- `componentized:secrets/store@0.1.1`: the store the gated store wraps

Exports:

- `componentized:secrets/store@0.1.1`: store gated by the latch
