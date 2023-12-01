

use std::sync::Arc;
use std::sync::Mutex;

use crate::core_store::CoreStore;
use crate::identifier::*;
use crate::process_rack_impl::ProcessRackImpl;
use crate::value::*;
use crate::error::*;
use crate::broker::*;
use crate::process::*;
use crate::process_rack::*;

use crate::manifest_checker::*;

#[allow(unused)]
pub struct CoreBroker {
    manifest: Value,
    process_rack: ProcessRackImpl,
    core_store: CoreStore,
}

impl CoreBroker {

    pub fn new(manifest: Value) -> Result<CoreBroker, JuizError> {
        match check_corebroker_manifest(manifest) {
            Err(err) => return Err(err),
            Ok(manif) => return Ok(CoreBroker{manifest: manif, process_rack: ProcessRackImpl::new(), core_store: CoreStore::new()})
        }
    }

    pub fn push_process(&mut self, p: Arc<Mutex<dyn Process>>) -> Result<(), JuizError> {
        self.process_rack.push(p)
    }
}

impl<'a> Broker for CoreBroker {

    fn is_in_charge_for_process(&mut self, id: &Identifier) -> bool {
        self.process_rack.process(id).is_some()
    }

    fn call_process(&mut self, id: &Identifier, args: Value) -> Result<Value, JuizError> {
        match self.process_rack.process(id) {
            None => return Err(JuizError::ProcessCanNotFoundError{}),
            Some(p) => {
                match p.try_lock() {
                    Err(_e) => Err(JuizError::CoreBrokerCanNotLockProcessMutexError{}),
                    Ok(proc) => proc.call(args)
                }
            }
        }
    }

    #[allow(unused)]
    fn connect_process_to(&mut self, source_process_id: &Identifier, arg_name: &String, target_process_id: &Identifier) -> Result<Value, JuizError> {
        todo!()
    }

    fn create_process(&mut self, manifest: Value) -> Result<Arc<Mutex<dyn Process>>, JuizError> {
        let manifest_updated = check_broker_create_process_manifest(manifest)?;
        let type_name = manifest_updated.get("type_name").unwrap().as_str().unwrap();
        let name = type_name.to_string() + "0";
        self.core_store.process_factory(type_name)?.create_process(name.as_str(), manifest_updated)
    }
}