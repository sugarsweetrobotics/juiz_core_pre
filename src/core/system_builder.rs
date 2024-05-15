


pub mod system_builder {
    use std::{path::PathBuf, rc::Rc, sync::{mpsc, Arc, Mutex}};

    use anyhow::Context;

    use crate::{brokers::{broker_factories_wrapper::BrokerFactoriesWrapper, local::{local_broker::{create_local_broker_factory, BrokerSideSenderReceiverPair, ProxySideSenderReceiverPair}, local_broker_proxy::create_local_broker_proxy_factory}, local_broker::ByteSenderReceiverPair, BrokerFactory, BrokerProxy, BrokerProxyFactory}, connections::connection_builder::connection_builder, containers::{container_factory_wrapper::ContainerFactoryWrapper, container_process_factory_wrapper::ContainerProcessFactoryWrapper}, core::Plugin, ecs::{execution_context_holder::ExecutionContextHolder, execution_context_holder_factory::ExecutionContextHolderFactory, ExecutionContextFactory}, jvalue, processes::{capsule::{Capsule, CapsuleMap}, ProcessFactoryWrapper}, utils::{get_array, get_hashmap, juiz_lock, manifest_util::when_contains_do_mut, when_contains_do}, value::{obj_get, obj_get_str}, ContainerFactory, ContainerProcessFactory, JuizResult, ProcessFactory, System, Value};

    pub fn setup_plugins(system: &mut System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_plugins({}) called", manifest);

        let _ = when_contains_do_mut(manifest, "broker_factories", |v| {
            setup_broker_factories(system, v).with_context(||format!("system_builder::setup_broker_factories(manifest={manifest:}) failed."))
        })?;
        let _ = when_contains_do(manifest, "process_factories", |v| {
            setup_process_factories(system, v).with_context(||format!("system_builder::setup_process_factories(manifest={manifest:}) failed."))
        })?;
        let _ = when_contains_do(manifest, "container_factories", |v| {
            setup_container_factories(system, v).with_context(||format!("system_builder::setup_container_factories(manifest={manifest:}) failed."))
        })?;
        let _ = when_contains_do_mut(manifest, "components", |v| {
            setup_component_factories(system, v).with_context(||format!("system_builder::setup_component_factories(manifest={manifest:}) failed."))
        })?;
        let _ = when_contains_do(manifest, "ec_factories", |v| {
            setup_execution_context_factories(system, v).with_context(||format!("system_builder::setup_execution_context_factories(manifest={manifest:}) failed."))
        })?;
        
        log::trace!("system_builder::setup_plugins() exit");
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn plugin_name_to_file_name(name: &String) -> String {
        "lib".to_string() + name + ".dylib"
    }

    fn concat_dirname(v: &serde_json::Value, name: String) -> JuizResult<PathBuf> {
        Ok(PathBuf::from(obj_get_str(v, "path")?.to_string()).join(name))
    }

    fn plugin_path(name: &String, v: &Value) -> JuizResult<std::path::PathBuf> {
        concat_dirname(v, plugin_name_to_file_name(name))
    }

    unsafe fn load_factory<'a, T: 'static + ?Sized>(rc_plugin: Rc<Plugin>, symbol_name: &str) -> JuizResult<Arc<Mutex<T>>> {
        log::trace!("load_factory(symbol_name={symbol_name}) called");
        type SymbolType<'a, T> = libloading::Symbol<'a, unsafe extern "Rust" fn() -> JuizResult<Arc<Mutex<T>>>>;
        let symbol = rc_plugin.load_symbol::<SymbolType<'a, T>>(symbol_name.as_bytes())?;
        (symbol)().with_context(||format!("calling symbol '{symbol_name}'"))
    }

    fn setup_process_factories(system: &System, manifest: &serde_json::Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_process_factories() called");
        for (name, v) in get_hashmap(manifest)?.iter() {
            unsafe {
                register_process_factory(system, Rc::new(Plugin::load(plugin_path(name, v)?)?), "process_factory")?;
            }
        }
        Ok(())
    }

    fn setup_container_factories(system: &System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_container_factories() called");
        for (name, v) in get_hashmap(manifest)?.iter() {
            unsafe {
                let rc_plugin = Rc::new(Plugin::load(plugin_path(name, v)?)?);
                register_container_factory(system, rc_plugin, "container_factory")?;
                when_contains_do(v, "processes", |vv| {
                    for (name, value) in get_hashmap(vv)?.iter() {
                        register_container_process_factory(system, Rc::new(Plugin::load(plugin_path(name, value)?)?), "container_process_factory")?;
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    fn setup_component_factories(system: &System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_component_factories({manifest:?}) called");
        for (name, v) in get_hashmap(manifest)?.iter() {

            log::debug!("Loading Component (name={name:})....");
            unsafe {
                type ComponentProfileFunctionSymbolType<'a> = libloading::Symbol<'a, unsafe extern "Rust" fn() -> Value>;
                let rc_plugin = Rc::new(Plugin::load(plugin_path(name, v)?)?);
                //log::trace!("!!!!!!!ComponentName: {}", (name.to_owned() + "::container_factory"));
                let symbol = rc_plugin.load_symbol::<ComponentProfileFunctionSymbolType>(b"component_profile")?;
                let cp = (symbol)();//.with_context(||format!("calling symbol 'container_factory'. arg is {manifest:}"))?;
                for container_profile in get_array(&cp["containers"])?.iter() {
                    let container_type_name = obj_get_str(container_profile, "type_name")?;
                    log::debug!(" - Loading Container ({container_type_name:}) Loading....");
                    register_container_factory(system, rc_plugin.clone(), obj_get_str(container_profile, "factory")?).context("ContainerFactoryWrapper::new()")?;
                    when_contains_do(container_profile, "processes", |vv| {
                        for v in get_array(vv)?.iter() {
                            let container_process_type_name = obj_get_str(v, "type_name")?;
                            log::debug!(" - Loading ContainerProcess ({container_process_type_name:}:{container_type_name}) Loading....");
                            register_container_process_factory(system, rc_plugin.clone(), obj_get_str(v, "factory")?)?;
                        }
                        return Ok(());
                    })?;
                }
                for process_profile in get_array(&cp["processes"])?.iter() {
                    let process_type_name = obj_get_str(process_profile, "type_name")?;
                    log::debug!(" - Loading Process ({process_type_name:}) Loading....");
                    register_process_factory(system, rc_plugin.clone(), obj_get_str(process_profile, "factory")?).context("ProcessFactoryWrapper::new()")?;
                }
            }
        }
        
        Ok(())
    }

    fn setup_execution_context_factories(system: &System, manifest: &serde_json::Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_execution_context_factories() called");
        for (name, value) in get_hashmap(manifest)?.iter() {
            log::debug!("Loading ExecutionContext (name={name:})....");
            let plugin_filename = concat_dirname(value, plugin_name_to_file_name(name))?;
            let cpf;
            unsafe {
                type ECFactorySymbolType<'a> = libloading::Symbol<'a, unsafe extern "Rust" fn() -> JuizResult<Arc<Mutex<dyn ExecutionContextFactory>>>>;
                let plugin: Plugin = Plugin::load(plugin_filename)?;
                {
                    let symbol = plugin.load_symbol::<ECFactorySymbolType>(b"execution_context_factory")?;
                    cpf = (symbol)().with_context(||format!("calling symbol 'execution_context_factory'. arg is {manifest:}"))?;
                    let ccpf = juiz_lock(&cpf)?;
                    log::debug!("ExecutionContextFactory (type_name={:?}) created.", ccpf.type_name());
                }
                system.core_broker().lock().unwrap().store_mut().ecs.register_factory(ExecutionContextHolderFactory::new(plugin, cpf)?)?;
            }
        }
        Ok(())
    }

    pub fn setup_local_broker_factory(system: &mut System) -> JuizResult<()> {
        log::trace!("system_builder::setup_local_broker_factory() called");
        let (c_sender, c_receiver) = mpsc::channel::<CapsuleMap>();
        let (p_sender, p_receiver) = mpsc::channel::<Arc<Mutex<Capsule>>>();
        let (_c_b_sender, c_b_receiver) = mpsc::channel::<Vec<u8>>();
        let (p_b_sender, _p_b_receiver) = mpsc::channel::<Vec<u8>>();
        let lbf = create_local_broker_factory(system.core_broker().clone(), Arc::new(Mutex::new(BrokerSideSenderReceiverPair(p_sender, c_receiver))), Arc::new(Mutex::new(ByteSenderReceiverPair(p_b_sender, c_b_receiver))))?;
        let lbpf = create_local_broker_proxy_factory(Arc::new(Mutex::new(ProxySideSenderReceiverPair(c_sender, p_receiver))))?;
        //juiz_lock(system.core_broker())?.store_mut().broker_proxies.register_factory(lbpf.clone())?;
        let _wrapper = system.register_broker_factories_wrapper(BrokerFactoriesWrapper::new(None, lbf, lbpf)?)?;
        Ok(())
    }

    fn setup_broker_factories(system: &mut System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_broker_factories() called");
        for (name, v) in get_hashmap(manifest)?.iter() {
            log::debug!("Loading Broker (name={name:})....");
            let plugin_filename = concat_dirname(v, plugin_name_to_file_name(name))?;
            let bf;
            let bpf;
            unsafe {
                type BrokerFactorySymbolType<'a> = libloading::Symbol<'a, unsafe extern "Rust" fn(Arc<Mutex<dyn BrokerProxy>>) -> JuizResult<Arc<Mutex<dyn BrokerFactory>>>>;
                type BrokerProxyFactorySymbolType<'a> = libloading::Symbol<'a, unsafe extern "Rust" fn() -> JuizResult<Arc<Mutex<dyn BrokerProxyFactory>>>>;
                let plugin: Plugin = Plugin::load(plugin_filename)?;
                {
                    // println!("!!!!!!!ContainerName: {}", (name.to_owned() + "::container_factory"));
                    //let symbol = plugin.load_symbol::<ContainerFactorySymbolType>((name.to_owned() + "::container_factory").as_bytes())?;
                    let symbol_bf = plugin.load_symbol::<BrokerFactorySymbolType>(b"broker_factory")?;
                    bf = (symbol_bf)(system.core_broker().clone()).with_context(||format!("calling symbol 'broker_factory'. arg is {manifest:}"))?;
                    log::debug!("BrokerFactory (type_name={:?}) created.", juiz_lock(&bf)?.type_name());
                    let symbol_bpf = plugin.load_symbol::<BrokerProxyFactorySymbolType>(b"broker_proxy_factory")?;
                    bpf = (symbol_bpf)().with_context(||format!("calling symbol 'broker_proxy_factory'. arg is {manifest:}"))?;
                    log::debug!("BrokerProxyFactory (type_name={:?}) created.", juiz_lock(&bpf)?.type_name());
                }
                system.register_broker_factories_wrapper(BrokerFactoriesWrapper::new(Some(plugin), bf, bpf)?)?;
            }
        }
        Ok(())
    }

    pub fn setup_processes(system: &System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_processes() called");
        for p in get_array(manifest)?.iter() {
            juiz_lock(system.core_broker())?.create_process_ref(p.clone())?;
        } 
        Ok(())
    }

    pub fn setup_containers(system: &System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_containers() called");
        for container_manifest in get_array(manifest)?.iter() {
            let container = juiz_lock(system.core_broker())?.create_container_ref(container_manifest.clone())?;
            let _ = when_contains_do(container_manifest, "processes", |v| {
                for p in get_array(v)?.iter() {
                    juiz_lock(system.core_broker())?.create_container_process_ref(Arc::clone(&container), p.clone())?;
                }
                Ok(())
            })?;
        } 
        Ok(())
    }

    pub fn setup_local_broker(system: &mut System) -> JuizResult<()> {
        log::trace!("system_builder::setup_local_broker() called");
        let local_broker = system.create_broker(&jvalue!({
            "type_name": "local",
            "name": "local"
        })).context("system.create_broker() failed in system_builder::setup_local_broker()")?;
        system.register_broker(local_broker)?;

        //let http_broker = HTTPBroker::new(
        //    system.core_broker().clone(), "http_broker_0")?;
        //system.register_broker(http_broker)?;
        Ok(())
    }

    pub fn setup_brokers(system: &mut System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_brokers() called");
        for p in get_array(manifest)?.iter() {
            let _ = system.create_broker(&p)?;
        }
        Ok(())
    }

    pub fn cleanup_brokers(system: &mut System) -> JuizResult<()> {
        log::trace!("system_builder::cleanup_ecs() called");
        system.cleanup_brokers()
    }

    pub fn setup_ecs(system: &System, manifest: &Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_ecs({manifest}) called");
        for p in get_array(manifest)?.iter() {
            let ec = juiz_lock(system.core_broker())?.create_ec_ref(p.clone())?;
            match obj_get(p, "bind") {
                Err(_) => {},
                Ok(binds_obj) => {
                    for b in get_array(binds_obj)?.iter() {
                        setup_ec_bind(system, Arc::clone(&ec), b)?;
                    }
                }
            };
        } 
        Ok(())
    }

    pub fn cleanup_ecs(system: &System) -> JuizResult<()> {
        log::trace!("system_builder::cleanup_ecs() called");
        juiz_lock(system.core_broker())?.cleanup_ecs()
    }

    fn setup_ec_bind(system: &System, ec: Arc<Mutex<ExecutionContextHolder>>, bind_info: &Value) -> JuizResult<()> {
        let process_info = obj_get(bind_info, "target")?;
        let target_process = system.any_process_from_manifest(process_info)?;
        juiz_lock(&ec)?.bind(target_process)
    }

    pub fn setup_connections(system: &System, manifest: &serde_json::Value) -> JuizResult<()> {
        log::trace!("system_builder::setup_connections() called");
        for c in get_array(manifest)?.iter() {
            connection_builder::create_connection(system, &c).context("connection_builder::create_connections faled in system_builder::setup_connections()")?;
        } 
        Ok(())
    }

        
    ///
    unsafe fn register_process_factory(system: &System, rc_plugin: Rc<Plugin>, symbol_name: &str) -> JuizResult<Arc<Mutex<dyn ProcessFactory>>> {
        log::trace!("register_process_factory(symbol_name={symbol_name}) called");
        let pf = load_factory::<dyn ProcessFactory>(rc_plugin.clone(), symbol_name)?;
        system.core_broker().lock().unwrap().store_mut().processes.register_factory(ProcessFactoryWrapper::new(rc_plugin.clone(), pf)?)
    }

    ///
    unsafe fn register_container_factory(system: &System, rc_plugin: Rc<Plugin>, symbol_name: &str) -> JuizResult<Arc<Mutex<dyn ContainerFactory>>> {
        log::trace!("register_container_factory(symbol_name={symbol_name}) called");
        let cf = load_factory::<dyn ContainerFactory>(rc_plugin.clone(), symbol_name)?;
        system.core_broker().lock().unwrap().store_mut().containers.register_factory(ContainerFactoryWrapper::new(rc_plugin.clone(), cf)?)
    }

    ///
    unsafe fn register_container_process_factory(system: &System, rc_plugin: Rc<Plugin>, symbol_name: &str) -> JuizResult<Arc<Mutex<dyn ContainerProcessFactory>>> {
        log::trace!("register_container_process_factory(symbol_name={symbol_name}) called");
        let cpf = load_factory::<dyn ContainerProcessFactory>(rc_plugin.clone(), symbol_name)?;
        system.core_broker().lock().unwrap().store_mut().container_processes.register_factory(ContainerProcessFactoryWrapper::new(rc_plugin.clone(), cpf)?)
    }
}