

use crate::{JuizResult, utils::juiz_lock, JuizError, Value};
use std::sync::{Arc, Mutex};
use crate::{core::Plugin, brokers::{Broker, BrokerProxy, BrokerFactory, BrokerProxyFactory}};

#[allow(dead_code)]
pub struct BrokerFactoriesWrapper {
    type_name: String,
    plugin: Option<Plugin>, 
    pub broker_factory: Arc<Mutex<dyn BrokerFactory>>,
    pub broker_proxy_factory: Arc<Mutex<dyn BrokerProxyFactory>>,
}

impl BrokerFactoriesWrapper {

    pub fn new(plugin: Option<Plugin>, broker_factory: Arc<Mutex<dyn BrokerFactory>>, broker_proxy_factory: Arc<Mutex<dyn BrokerProxyFactory>>) -> JuizResult<Arc<Mutex<BrokerFactoriesWrapper>>> {
        let type_name_bf = juiz_lock(&broker_factory)?.type_name().to_string();
        let type_name_bpf = juiz_lock(&broker_proxy_factory)?.type_name().to_string();
        if type_name_bf != type_name_bpf {
            return Err(anyhow::Error::from(JuizError::BrokerFactoryAndBrokerProxyFactoryWithDifferentTypeIsRegisteredError{type_name_bf: type_name_bf, type_name_bpf: type_name_bpf}))
        }

        Ok(Arc::new(Mutex::new(BrokerFactoriesWrapper{
            type_name: type_name_bpf.to_string(),
            plugin,
            broker_factory,
            broker_proxy_factory
        })))
    }

    pub fn profile_full(&self) -> JuizResult<Value> {
        juiz_lock(&self.broker_factory)?.profile_full()
    }

    pub fn type_name(&self) -> &str {
        &self.type_name.as_str()
    }

    pub fn create_broker(&self, manifest: &Value) -> JuizResult<Arc<Mutex<dyn Broker>>> {
        juiz_lock(&self.broker_factory)?.create_broker(manifest.clone())
    }

    pub fn create_broker_proxy(&self, manifest: &Value) -> JuizResult<Arc<Mutex<dyn BrokerProxy>>> {
        juiz_lock(&self.broker_proxy_factory)?.create_broker_proxy(manifest.clone())
    }
}