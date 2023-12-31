use std::sync::{Arc, Mutex};

use crate::{Process, JuizObject, processes::{process_impl::ProcessImpl, argument::Argument, Output}, Identifier, Value, JuizResult, Container, utils::{juiz_lock, check_process_manifest}, JuizError, jvalue, value::{obj_get_str, obj_merge}, object::{ObjectCore, JuizObjectClass, JuizObjectCoreHolder}};

use super::container_impl::ContainerImpl;
//use crate::containers::container_process_impl::JuizObjectClass::ContainerProcess;

use crate::processes::process_impl::{FunctionTrait, FunctionType};

//pub type ContainerProcessFunction<T>=dyn Fn (&mut Box<T>, Value) -> JuizResult<Value> + 'static;
pub type ContainerFunctionTrait<T>=dyn Fn(&mut Box<T>, Vec<Argument>) -> JuizResult<Output> + 'static;
pub type ContainerFunctionType<T>=fn (&mut Box<T>, Vec<Argument>) -> JuizResult<Output>;

#[allow(dead_code)]
pub struct ContainerProcessImpl<T: 'static> {
    core: ObjectCore,
    process: ProcessImpl,
    pub container: Arc<Mutex<dyn Container>>,
    container_identifier: Identifier,
    function: ContainerFunctionType<T>,
}

impl<T: 'static> ContainerProcessImpl<T> {

    pub fn new<'a> (manif: Value, container: Arc<Mutex<dyn Container>>, function: ContainerFunctionType<T>) -> JuizResult<Self> {
        log::trace!("ContainerProcessImpl::new(manifest={}) called", manif);
        //let identifier = create_identifier_from_manifest("ContainerProcess", &manif)?;
        let manifest = check_process_manifest(manif)?;
        let container_clone = Arc::clone(&container);
        let container_identifier = juiz_lock(&container)?.identifier().clone();
        let proc = ProcessImpl::clousure_new_with_class_name(JuizObjectClass::ContainerProcess("ProcessImpl"), manifest.clone(), Box::new(move |args| {
            let mut locked_container = juiz_lock(&container)?;
            match locked_container.downcast_mut::<ContainerImpl<T>>() {
                None => Err(anyhow::Error::from(JuizError::ContainerDowncastingError{identifier: locked_container.identifier().clone()})),
                Some(container_impl) => {
                    Ok((function)(&mut container_impl.t, args)?)
                }
            }
            
        }))?;
        
        let type_name = obj_get_str(&manifest, "type_name")?;
        let object_name = obj_get_str(&manifest, "name")?;
        Ok(  
            (
                move || ContainerProcessImpl::<T>{
                    core: ObjectCore::create(JuizObjectClass::ContainerProcess("ContainerProcessImpl"), 
                        type_name, object_name),
                    container_identifier,
                    container: container_clone,
                    process: proc,
                    function,
                }
            )()
        )
    }

    
}

impl<T: 'static> JuizObjectCoreHolder for ContainerProcessImpl<T> {
    fn core(&self) -> &ObjectCore {
        &self.core
    }
}

impl<T: 'static> JuizObject for ContainerProcessImpl<T> {
    fn profile_full(&self) -> JuizResult<Value> {
        obj_merge(self.process.profile_full()?, &jvalue!({
            "container_identifier": self.container_identifier
        }))
    }
}

impl<T: 'static> Process for ContainerProcessImpl<T> {


    fn manifest(&self) -> &crate::Value {
        self.process.manifest()
    }

    fn call(&self, args: crate::Value) -> crate::JuizResult<Output> {
        self.process.call(args)
    }

    fn is_updated(& self) -> crate::JuizResult<bool> {
        self.process.is_updated()
    }

    fn is_updated_exclude(& self, caller_id: &crate::Identifier) -> crate::JuizResult<bool> {
        self.process.is_updated_exclude(caller_id)
    }


    fn invoke<'b>(&self) -> crate::JuizResult<Output> {
        self.process.invoke()
    }

    fn invoke_exclude<'b>(&self, arg_name: &String, value: Output) -> JuizResult<Output> {
        self.process.invoke_exclude(arg_name, value)
    }

    fn execute(&self) -> JuizResult<Output> {
        self.process.execute()
    }

    fn push_by(&self, arg_name: &String, value: &Output) -> crate::JuizResult<Output> {
        self.process.push_by(arg_name, value)
    }

    fn get_output(&self) -> Option<Output> {
        self.process.get_output()
    }

    fn notify_connected_from<'b>(&'b mut self, source: Arc<Mutex<dyn Process>>, connecting_arg: &String, connection_manifest: crate::Value) -> crate::JuizResult<crate::Value> {
        self.process.notify_connected_from(source, connecting_arg, connection_manifest)
    }

    fn try_connect_to(&mut self, target: Arc<Mutex<dyn Process>>, connect_arg_to: &String, connection_manifest: crate::Value) -> crate::JuizResult<crate::Value> {
        self.process.try_connect_to(target, connect_arg_to, connection_manifest)
    }

    fn source_connections(&self) -> JuizResult<Vec<&Box<dyn crate::connections::SourceConnection>>> {
        self.process.source_connections()
    }

    fn destination_connections(&self) -> JuizResult<Vec<&Box<dyn crate::connections::DestinationConnection>>> {
        self.process.destination_connections()
    }
}




unsafe impl<T: 'static> Send for ContainerProcessImpl<T> {
}