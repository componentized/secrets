# Secrets components <!-- omit in toc -->

A collection of utility components for sensitive values.

- [Components](#components)
- [Build](#build)
- [Community](#community)
  - [Code of Conduct](#code-of-conduct)
  - [Communication](#communication)
  - [Contributing](#contributing)
- [Acknowledgements](#acknowledgements)
- [License](#license)


## Components

- [`empty`](./components/empty/)
- [`from-config`](./components/from-config/)

Access control, a [`gate`](./components/gate/) consults a latch before each `get`, see [`latch.wit`](./wit/latch.wit):

- [`gate`](./components/gate/)
- [`latch-defer-all`](./components/latch-defer-all/)
- [`latch-deny-all`](./components/latch-deny-all/)
- [`latch-key`](./components/latch-key/)
- [`latch-n2`](./components/latch-n2/), [`latch-n3`](./components/latch-n3/), [`latch-n4`](./components/latch-n4/), [`latch-n5`](./components/latch-n5/), aggregate 2 to 5 latches into one

## Build

A [dev container](https://containers.dev) is available that contains the necessary tools and configuration out of the box.

Prereqs:
- a rust toolchain
- [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall)

```sh
make components
```

To test the components:

```sh
make test
```

Each store is plugged into [`test-consumer`](./components/test-consumer/), which calls the store the way a consumer does, e.g. [`test-empty`](./components/test-empty/), and each component's tests, in its `tests` directory, run the composition with wasmtime, with the [`test-harness`](./crates/test-harness/). The `gate` is tested between the consumer and a store, [`test-gate`](./components/test-gate/), with a latch the tests provide. The latches are called by the tests directly.

## Community

### Code of Conduct

The Componentized project follow the [Contributor Covenant Code of Conduct](./CODE_OF_CONDUCT.md). In short, be kind and treat others with respect.

### Communication

General discussion and questions about the project can occur in the project's [GitHub discussions](https://github.com/orgs/componentized/discussions).

### Contributing

The Componentized project team welcomes contributions from the community. A contributor license agreement (CLA) is not required. You own full rights to your contribution and agree to license the work to the community under the Apache License v2.0, via a [Developer Certificate of Origin (DCO)](https://developercertificate.org). For more detailed information, refer to [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgements

This project was conceived in discussion between [Mark Fisher](https://github.com/markfisher) and [Scott Andrews](https://github.com/scothis).

## License

Apache License v2.0: see [LICENSE](./LICENSE) for details.
