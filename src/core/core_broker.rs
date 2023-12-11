

use std::sync::Arc;
use std::sync::Mutex;


use anyhow::Context;

use crate::Container;
use crate::ContainerProcess;
use crate::utils::check_corebroker_manifest;
use crate::utils::juiz_lock;
use crate::utils::manifest_util::type_name;
use crate::value::obj_get_str;
use crate::{Value, jvalue, Process, BrokerProxy, Identifier, JuizError, JuizResult, core::core_store::CoreStore};

use crate::connections::connect;

#[allow(unused)]
pub struct CoreBroker {
    manifest: Value,
    core_store: CoreStore,
}

impl CoreBroker {

    pub fn new(manifest: Value) -> Result<CoreBroker, JuizError> {
        Ok(CoreBroker{
            manifest: check_corebroker_manifest(manifest)?,
            core_store: CoreStore::new()
        })
    }

    pub fn store(&self) -> &CoreStore {
        &self.core_store
    }

    pub fn store_mut(&mut self) -> &mut CoreStore {
        &mut self.core_store
    }

    fn gen_identifier(&self, mut manifest: Value) -> JuizResult<Value> {
        let name = obj_get_str(&manifest, "name")?;
        let type_name = obj_get_str(&manifest, "type_name")?;
        let id = "core://" .to_string()+ name + ":" + type_name;
        manifest.as_object_mut().unwrap().insert("identifier".to_string(), jvalue!(id));
        return Ok(manifest);
    }

    fn gen_name_if_noname(&self, mut manifest: Value) -> JuizResult<Value> {
        if manifest.get("name").is_some() {
            return Ok(manifest);
        }
        let name = type_name(&manifest)?.to_string() + "0";
        manifest.as_object_mut().unwrap().insert("name".to_string(), jvalue!(name));
        return Ok(manifest);
    }

    fn check_has_type_name(&self, manifest: Value) -> JuizResult<Value> {
        let manifest_updated = manifest.clone();
        // let _ = obj_get_str(&manifest,"name")?;
        let _ = obj_get_str(&manifest, "type_name")?;
        return Ok(manifest_updated)
    }

    fn precreate_check<'b>(&'b self, manifest: Value) -> JuizResult<Value> {
        self.gen_identifier(self.gen_name_if_noname(self.check_has_type_name(manifest)?)?)
    }

    //pub fn process_ref(&self, id: &Identifier) -> JuizResult<Arc<Mutex<dyn Process>>> {
    //    self.store().process(id)
    //}

    pub fn create_process_ref(&mut self, manifest: Value) -> JuizResult<Arc<Mutex<dyn Process>>> {
        log::trace!("CoreBroker::create_process(manifest={}) called", manifest);
        let arc_pf = self.core_store.process_factory(type_name(&manifest)?)?;
        let p = juiz_lock(arc_pf)?.create_process(self.precreate_check(manifest)?)?;
        self.store_mut().register_process(p)
    }

    pub fn create_container_ref(&mut self, manifest: Value) -> JuizResult<Arc<Mutex<dyn Container>>> {
        log::trace!("CoreBroker::create_container(manifest={}) called", manifest);
        let arc_pf = self.core_store.container_factory(type_name(&manifest)?)?;
        let p = juiz_lock(arc_pf)?.create_container(self.precreate_check(manifest)?)?;
        self.store_mut().register_container(p)
    }

    pub fn create_container_process_ref(&mut self, container: Arc<Mutex<dyn Container>>, manifest: Value) -> JuizResult<Arc<Mutex<dyn ContainerProcess>>> {
        log::trace!("CoreBroker::create_container_process(manifest={}) called", manifest);
        let typ_name = type_name(&manifest)?;
        let arc_pf = self.core_store.container_process_factory(typ_name).with_context(||format!("CoreBroker::create_container_process({})", typ_name))?;
        let p = juiz_lock(arc_pf)?.create_container_process(Arc::clone(&container), self.precreate_check(manifest)?)?;
        self.store_mut().register_container_process(p)
    }



}

impl BrokerProxy for CoreBroker {

    fn is_in_charge_for_process(&self, id: &Identifier) -> JuizResult<bool> {
        Ok(self.store().process(id).is_ok())
    }

    fn call_process(&self, id: &Identifier, args: Value) -> JuizResult<Value> {
        juiz_lock(&self.store().process(id)?)?.call(args)
    }

    fn connect_process_to(&mut self, source_process_id: &Identifier, arg_name: &String, target_process_id: &Identifier, manifest: Value) -> JuizResult<Value> {
        connect(self.store().process(source_process_id)?, self.store().process(target_process_id)?, arg_name, manifest)
    }

    fn execute_process(&self, id: &Identifier) -> JuizResult<Value> {
        juiz_lock(&self.store().process(id)?).with_context(||format!("locking process(id={id:}) in CoreBroker::execute_process() function"))?.execute()
    }

    fn profile_full(&self) -> JuizResult<Value> {
        Ok(jvalue!({
            "core_store" : self.core_store.profile_full()?
        }))
    }
}

unsafe impl Send for CoreBroker {

}