//! The harness of the components' tests, in each component's `tests` directory. Runs a component
//! with wasmtime, as the host of the `test-host` world: a composition of `test-consumer` through
//! its probe, see `components/test-consumer`, or a latch directly.
//!
//! Each store is plugged into `test-consumer`, which calls the store as a consumer does, e.g.
//! `test-empty`. The gate is tested between the consumer and a store, `test-gate`, with a latch the
//! host provides. The latches are called by the host directly.

pub mod latch_n;

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, PoisonError};

use wasmtime::component::{Component, HasData, HasSelf, Instance, InstancePre, Linker};
use wasmtime::{Config, Engine, Store};

pub use bindings::componentized::secrets::latch::{
    Decision, ErrorCode as LatchError, GetArgs, Operation, StoreOperation,
};
pub use bindings::componentized::secrets::store::{ErrorCode, Secret};
pub use bindings::exports::componentized::secrets_components::probe::InitCall;
pub use bindings::wasi::config::store::Error as ConfigError;
pub use bindings::wasi::logging::logging::Level;

/// The latch a latch component exports, its types are distinct from the latch the host provides.
pub mod exported {
    pub use super::bindings::exports::componentized::secrets::latch::{
        Decision, ErrorCode as LatchError, GetArgs, Operation, StoreOperation,
    };
}

mod bindings {
    wasmtime::component::bindgen!({
        path: "../../components/wit",
        world: "test-host",
        exports: { default: async },
        additional_derives: [PartialEq],
    });
}

/// The host of a component, providing the imports the components under test use.
pub struct Host {
    /// The config values, or the error getting them.
    pub config: Result<HashMap<String, String>, ConfigError>,
    /// The number of calls to `wasi:config/store.get-all`.
    pub get_all_calls: usize,
    /// The messages logged with `wasi:logging`.
    pub logs: Vec<Log>,
    /// The latch a gate consults.
    pub latch: FakeLatch,
    /// The latches a `latch-n` aggregates, `latch0` to `latch4`.
    pub latches: [FakeLatch; 5],
    /// The calls to `latches`, in order, e.g. `latch1.authorize`.
    pub latch_calls: Vec<String>,
}

impl Default for Host {
    /// No config values, and a latch that defers everything.
    fn default() -> Self {
        Self {
            config: Ok(HashMap::new()),
            get_all_calls: 0,
            logs: Vec::new(),
            latch: FakeLatch::default(),
            latches: Default::default(),
            latch_calls: Vec::new(),
        }
    }
}

/// A message logged with `wasi:logging`.
#[derive(Clone, Debug, PartialEq)]
pub struct Log {
    pub level: Level,
    pub context: String,
    pub message: String,
}

/// A latch that returns the decision it is given, recording how it is called.
pub struct FakeLatch {
    /// The result of each `authorize`.
    pub authorize: Result<Decision, LatchError>,
    /// The result of each `observe-decision`.
    pub observe: Result<(), LatchError>,
    /// The operations `authorize` was called with.
    pub authorized: Vec<Operation>,
    /// The decisions and operations `observe-decision` was called with.
    pub observed: Vec<(Decision, Operation)>,
}

impl Default for FakeLatch {
    fn default() -> Self {
        Self {
            authorize: Ok(Decision::Deferred),
            observe: Ok(()),
            authorized: Vec::new(),
            observed: Vec::new(),
        }
    }
}

impl bindings::wasi::config::store::Host for Host {
    fn get(&mut self, key: String) -> Result<Option<String>, ConfigError> {
        self.config.clone().map(|config| config.get(&key).cloned())
    }

    fn get_all(&mut self) -> Result<Vec<(String, String)>, ConfigError> {
        self.get_all_calls += 1;
        self.config
            .clone()
            .map(|config| config.into_iter().collect())
    }
}

impl bindings::wasi::logging::logging::Host for Host {
    fn log(&mut self, level: Level, context: String, message: String) {
        self.logs.push(Log {
            level,
            context,
            message,
        });
    }
}

/// The latch a gate imports, as `componentized:secrets/latch`, rather than one of `Host::latches`.
const GATE_LATCH: usize = usize::MAX;

/// A [`FakeLatch`] of the host, `Host::latch` for [`GATE_LATCH`], otherwise `Host::latches[I]`
/// imported as `latchI`, whose calls are recorded in `Host::latch_calls`.
struct FakeLatchAccess<'a, const I: usize>(&'a mut Host);

impl<const I: usize> FakeLatchAccess<'_, I> {
    fn latch(&mut self, call: &str) -> &mut FakeLatch {
        if I == GATE_LATCH {
            return &mut self.0.latch;
        }
        self.0.latch_calls.push(format!("latch{I}.{call}"));
        &mut self.0.latches[I]
    }
}

impl<const I: usize> bindings::componentized::secrets::latch::Host for FakeLatchAccess<'_, I> {
    fn authorize(&mut self, operation: Operation) -> Result<Decision, LatchError> {
        let latch = self.latch("authorize");
        latch.authorized.push(operation);
        latch.authorize.clone()
    }

    fn observe_decision(
        &mut self,
        final_decision: Decision,
        operation: Operation,
    ) -> Result<(), LatchError> {
        let latch = self.latch("observe-decision");
        latch.observed.push((final_decision, operation));
        latch.observe.clone()
    }
}

struct FakeLatchData<const I: usize>;

impl<const I: usize> HasData for FakeLatchData<I> {
    type Data<'a> = FakeLatchAccess<'a, I>;
}

/// Adds the fake latch `I` to the linker as the import `name`.
fn add_fake_latch<const I: usize>(linker: &mut Linker<Host>, name: &str) -> wasmtime::Result<()> {
    bindings::componentized::secrets::latch::add_to_linker_instance::<Host, FakeLatchData<I>>(
        &mut linker.instance(name)?,
        |host| FakeLatchAccess(host),
    )
}

/// The operation to get the secret of the key.
pub fn get(key: &str) -> Operation {
    Operation::Store(StoreOperation::Get(GetArgs {
        key: key.to_string(),
    }))
}

/// Builds the component with `make components/<component>`, so a test run on its own, e.g. with
/// `cargo test`, runs the latest sources. Each component is built once a test run, one at a time,
/// make builds the components a composition plugs together, which compositions share.
///
/// `make test` builds every component before the tests and sets `COMPONENTS_BUILT=true`, the
/// components are not built again.
fn build(root: &Path, component: &str) -> wasmtime::Result<()> {
    static BUILT: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

    if std::env::var("COMPONENTS_BUILT").is_ok_and(|built| built == "true") {
        return Ok(());
    }
    let mut built = BUILT.lock().unwrap_or_else(PoisonError::into_inner);
    if built.contains(component) {
        return Ok(());
    }
    let target = format!("components/{component}");
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
    built.insert(component.to_string());
    Ok(())
}

/// Instantiates the release build of the component from target/components, built first, see
/// [`build`]. Imports the host doesn't provide trap when called, e.g. the types-only
/// `componentized:secrets/store` a composition with a gate imports for its latch.
async fn instantiate(
    component: &str,
    host: Host,
) -> wasmtime::Result<(Store<Host>, InstancePre<Host>, Instance)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    build(&root, component)?;
    let mut config = Config::new();
    config.wasm_component_model_async(true);
    // the named latch imports of a `latch-n`, e.g. `import latch0: componentized:secrets/latch`
    config.wasm_component_model_implements(true);
    let engine = Engine::new(&config)?;
    let path = root
        .join("target/components")
        .join(component)
        .join(format!("{component}.wasm"));
    let wasm = Component::from_file(&engine, &path)
        .map_err(|err| err.context(path.display().to_string()))?;
    let mut linker = Linker::new(&engine);
    bindings::wasi::config::store::add_to_linker::<_, HasSelf<Host>>(&mut linker, |host| host)?;
    bindings::wasi::logging::logging::add_to_linker::<_, HasSelf<Host>>(&mut linker, |host| host)?;
    add_fake_latch::<GATE_LATCH>(&mut linker, "componentized:secrets/latch@0.1.2-dev")?;
    add_fake_latch::<0>(&mut linker, "latch0")?;
    add_fake_latch::<1>(&mut linker, "latch1")?;
    add_fake_latch::<2>(&mut linker, "latch2")?;
    add_fake_latch::<3>(&mut linker, "latch3")?;
    add_fake_latch::<4>(&mut linker, "latch4")?;
    linker.define_unknown_imports_as_traps(&wasm)?;
    let mut store = Store::new(&engine, host);
    let pre = linker.instantiate_pre(&wasm)?;
    let instance = pre.instantiate_async(&mut store).await?;
    Ok((store, pre, instance))
}

/// An instance of a composition of `test-consumer`, e.g. `test-empty`.
pub struct Consumer {
    pub store: Store<Host>,
    probe: bindings::exports::componentized::secrets_components::probe::Guest,
}

impl Consumer {
    pub async fn new(composition: &str, host: Host) -> wasmtime::Result<Self> {
        let (mut store, pre, instance) = instantiate(composition, host).await?;
        let probe =
            bindings::exports::componentized::secrets_components::probe::GuestIndices::new(&pre)?
                .load(&mut store, &instance)?;
        Ok(Self { store, probe })
    }

    /// Calls `store.init` through the consumer.
    pub async fn init(&mut self) -> wasmtime::Result<InitCall> {
        let probe = self.probe.clone();
        self.store
            .run_concurrent(async |accessor| probe.call_init(accessor).await)
            .await?
    }

    /// Calls `store.init` through the consumer twice, at the same time.
    pub async fn init_concurrently(&mut self) -> wasmtime::Result<(InitCall, InitCall)> {
        let probe = self.probe.clone();
        self.store
            .run_concurrent(async |accessor| {
                let (first, second) =
                    tokio::join!(probe.call_init(accessor), probe.call_init(accessor));
                Ok((first?, second?))
            })
            .await?
    }

    /// Calls `store.get` through the consumer.
    pub async fn get(&mut self, key: &str) -> wasmtime::Result<Result<Secret, ErrorCode>> {
        self.probe.call_get(&mut self.store, key).await
    }
}

/// An instance of a latch component, e.g. `latch-key`.
pub struct Latch {
    pub store: Store<Host>,
    latch: bindings::exports::componentized::secrets::latch::Guest,
}

impl Latch {
    pub async fn new(component: &str, host: Host) -> wasmtime::Result<Self> {
        let (mut store, pre, instance) = instantiate(component, host).await?;
        let latch = bindings::exports::componentized::secrets::latch::GuestIndices::new(&pre)?
            .load(&mut store, &instance)?;
        Ok(Self { store, latch })
    }

    /// Calls `authorize` to get the secret of the key.
    pub async fn authorize_get(
        &mut self,
        key: &str,
    ) -> wasmtime::Result<Result<exported::Decision, exported::LatchError>> {
        self.latch
            .call_authorize(&mut self.store, &exported_get(key))
            .await
    }

    /// Calls `observe-decision` for getting the secret of the key.
    pub async fn observe_get(
        &mut self,
        decision: &exported::Decision,
        key: &str,
    ) -> wasmtime::Result<Result<(), exported::LatchError>> {
        self.latch
            .call_observe_decision(&mut self.store, decision, &exported_get(key))
            .await
    }
}

fn exported_get(key: &str) -> exported::Operation {
    exported::Operation::Store(exported::StoreOperation::Get(exported::GetArgs {
        key: key.to_string(),
    }))
}
