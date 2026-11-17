//! The tests of each component, run by `make test` after the components are built. Each component is
//! plugged into `test-consumer`, which calls the store as a consumer does, e.g. `test-empty`, and the
//! composition is run with wasmtime.

#[cfg(test)]
mod empty;
#[cfg(test)]
mod from_config;
#[cfg(test)]
mod host;
