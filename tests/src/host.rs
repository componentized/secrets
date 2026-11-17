//! Runs a composition of `test-consumer` with wasmtime, see `components/test-consumer`.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, PoisonError};

use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Config, Engine, Store};

pub use bindings::componentized::secrets::store::{ErrorCode, Secret};
pub use bindings::exports::componentized::secrets_components::probe::InitCall;

mod bindings {
    wasmtime::component::bindgen!({
        path: "../components/wit",
        world: "test-consumer",
        exports: { default: async },
    });
}

mod config {
    wasmtime::component::bindgen!({
        path: "../components/wit",
        world: "wasi:config/imports",
    });
}

pub use config::wasi::config::store::Error as ConfigError;

/// The host of a composition, providing `wasi:config` for the components that import it.
pub struct Host {
    /// The config values, or the error getting them.
    pub config: Result<HashMap<String, String>, ConfigError>,
    /// The number of calls to `wasi:config/store.get-all`.
    pub get_all_calls: usize,
}

impl Default for Host {
    /// No config values.
    fn default() -> Self {
        Self { config: Ok(HashMap::new()), get_all_calls: 0 }
    }
}

impl config::wasi::config::store::Host for Host {
    fn get(&mut self, key: String) -> Result<Option<String>, ConfigError> {
        self.config.clone().map(|config| config.get(&key).cloned())
    }

    fn get_all(&mut self) -> Result<Vec<(String, String)>, ConfigError> {
        self.get_all_calls += 1;
        self.config.clone().map(|config| config.into_iter().collect())
    }
}

/// Builds the composition with `make components/<composition>`, so a test run on its own, e.g. with
/// `cargo test`, runs the latest sources. Each composition is built once a test run, one at a time,
/// make builds the components a composition plugs together, which compositions share.
///
/// `make test` builds every component before the tests and sets `COMPONENTS_BUILT=true`, the
/// compositions are not built again.
fn build(root: &Path, composition: &str) -> wasmtime::Result<()> {
    static BUILT: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

    if std::env::var("COMPONENTS_BUILT").is_ok_and(|built| built == "true") {
        return Ok(());
    }
    let mut built = BUILT.lock().unwrap_or_else(PoisonError::into_inner);
    if built.contains(composition) {
        return Ok(());
    }
    let target = format!("components/{composition}");
    let output = Command::new("make")
        .arg("-C")
        .arg(root)
        .arg(&target)
        .output()
        .map_err(|err| wasmtime::Error::from(err).context(format!("running `make {target}`")))?;
    if !output.status.success() {
        wasmtime::bail!(
            "`make {target}` failed, {}\n{}{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    built.insert(composition.to_string());
    Ok(())
}

/// An instance of a composition, e.g. `test-empty`.
pub struct Consumer {
    pub store: Store<Host>,
    bindings: bindings::TestConsumer,
}

impl Consumer {
    /// Instantiates the release build of the composition from target/components, built first, see
    /// [`build`].
    pub async fn new(composition: &str, host: Host) -> wasmtime::Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        build(&root, composition)?;
        let mut config = Config::new();
        config.wasm_component_model_async(true);
        let engine = Engine::new(&config)?;
        let path = root
            .join("target/components")
            .join(composition)
            .join(format!("{composition}.wasm"));
        let component = Component::from_file(&engine, &path)
            .map_err(|err| err.context(path.display().to_string()))?;
        let mut linker = Linker::new(&engine);
        config::wasi::config::store::add_to_linker::<_, HasSelf<Host>>(&mut linker, |host| host)?;
        let mut store = Store::new(&engine, host);
        let bindings = bindings::TestConsumer::instantiate_async(&mut store, &component, &linker).await?;
        Ok(Self { store, bindings })
    }

    /// Calls `store.init` through the consumer.
    pub async fn init(&mut self) -> wasmtime::Result<InitCall> {
        let probe = self.bindings.componentized_secrets_components_probe().clone();
        self.store.run_concurrent(async |accessor| probe.call_init(accessor).await).await?
    }

    /// Calls `store.init` through the consumer twice, at the same time.
    pub async fn init_concurrently(&mut self) -> wasmtime::Result<(InitCall, InitCall)> {
        let probe = self.bindings.componentized_secrets_components_probe().clone();
        self.store
            .run_concurrent(async |accessor| {
                let (first, second) = tokio::join!(probe.call_init(accessor), probe.call_init(accessor));
                Ok((first?, second?))
            })
            .await?
    }

    /// Calls `store.get` through the consumer.
    pub async fn get(&mut self, key: &str) -> wasmtime::Result<Result<Secret, ErrorCode>> {
        let probe = self.bindings.componentized_secrets_components_probe();
        probe.call_get(&mut self.store, key).await
    }
}
