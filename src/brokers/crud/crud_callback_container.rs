use std::{collections::HashMap, path::PathBuf};

use crate::{brokers::broker_proxy::{BrokerBrokerProxy, ConnectionBrokerProxy, ContainerBrokerProxy, ContainerProcessBrokerProxy, ExecutionContextBrokerProxy, ProcessBrokerProxy, SystemBrokerProxy}, core::core_broker::CoreBrokerPtr, prelude::*};
use crate::value::{CapsuleMap, value_to_capsule};




pub type CBFnType = fn(CoreBrokerPtr, CapsuleMap)->JuizResult<CapsulePtr>;
pub type CallbackContainerType = HashMap<&'static str, CBFnType>;
pub type ClassCallbackContainerType = HashMap<&'static str, CallbackContainerType>;




pub(crate) fn create_callback_container() -> ClassCallbackContainerType {

    fn extract_create_parameter(args: CapsuleMap) -> JuizResult<Value> {
        log::warn!("extract_create_param({args:?})");
        let v = args.into();
        log::warn!(" - value: {v:?}");
        return Ok(v);
        //return args.get("map")?.try_into().or_else(|e|{Err(anyhow::Error::from(e))})
    }

    let mut create_cb_container = ClassCallbackContainerType::new();

    let mut process_callbacks = CallbackContainerType::new();
    process_callbacks.insert("create",  |cb, args| {
       Ok(cb.lock_mut()?.process_create(&extract_create_parameter(args)?)?.into())}
    );
    create_cb_container.insert("process", process_callbacks);


    let mut container_callbacks = CallbackContainerType::new();
    container_callbacks.insert("create",  |cb, args| {
       Ok(cb.lock_mut()?.container_create(&extract_create_parameter(args)?)?.into())}
    );
    create_cb_container.insert("container", container_callbacks);


    let mut container_process_callbacks = CallbackContainerType::new();
    container_process_callbacks.insert("create",  |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(cb.lock_mut()?.container_process_create(&id.clone(), &extract_create_parameter(args)?)?.into())}
    );
    create_cb_container.insert("container_process", container_process_callbacks);


    let mut ec_callbacks = CallbackContainerType::new();
    ec_callbacks.insert("create",  |cb, args| {
       Ok(cb.lock_mut()?.ec_create(&extract_create_parameter(args)?)?.into())}
    );
    create_cb_container.insert("execution_context", ec_callbacks);

    let mut connection_callbacks = CallbackContainerType::new();
    connection_callbacks.insert("create",  |cb, args| {
       Ok(cb.lock_mut()?.connection_create(extract_create_parameter(args)?)?.into())}
    );
    create_cb_container.insert("connection", connection_callbacks);

    create_cb_container
}


pub(crate) fn read_callback_container() -> ClassCallbackContainerType {
    let mut read_cb_container = ClassCallbackContainerType::new();
    let mut system_callbacks = CallbackContainerType::new();
    system_callbacks.insert("profile_full", |cb, _args| {
        log::trace!("system_callbacks['profile_full'] called with args {_args:?}");
        Ok(value_to_capsule(cb.lock()?.system_profile_full()?))
    });
    system_callbacks.insert("filesystem_list", |cb, _args| {
        log::trace!("system_callbacks['filesystem_list'] called with args {_args:?}");
        let param = _args.get_params();
        let mut path = ".".to_owned();
        if param.contains_key("path") {
            path = param.get("path").unwrap().clone();
        }
        Ok(value_to_capsule(cb.lock()?.system_filesystem_list(PathBuf::from(path))?))
    });
    system_callbacks.insert("add_subsystem", |cb, args| {
        log::trace!("system_callbacks['add_subsystem'] called with args {args:?}");
        let manif = args.get("profile")?.extract_value()?;
        Ok(value_to_capsule(cb.lock_mut()?.system_add_subsystem(manif)?))
    });

    read_cb_container.insert("system", system_callbacks);

    let mut broker_cbs = CallbackContainerType::new();
    broker_cbs.insert("profile_full", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.broker_profile_full(id)?))
    });
    broker_cbs.insert("list", |cb, _args| {
        Ok(value_to_capsule(cb.lock()?.broker_list()?))
    });
    read_cb_container.insert("broker", broker_cbs);


    let mut proc_cbs = CallbackContainerType::new();
    proc_cbs.insert("profile_full", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.process_profile_full(id)?))
    });
    proc_cbs.insert("list", |cb, _args| {
        Ok(value_to_capsule(cb.lock()?.process_list()?))
    });
    read_cb_container.insert("process", proc_cbs);


    let mut cont_cbs = CallbackContainerType::new();
    cont_cbs.insert("profile_full", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.container_profile_full(id)?))
    });
    cont_cbs.insert("list", |cb, _args| {
        Ok(value_to_capsule(cb.lock()?.container_list()?))
    });
    read_cb_container.insert("container", cont_cbs);
    
    let mut cpro_cbs = CallbackContainerType::new();
    cpro_cbs.insert("profile_full", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.container_process_profile_full(id)?))
    });
    cpro_cbs.insert("list", |cb, _args| {
        Ok(value_to_capsule(cb.lock()?.container_process_list()?))
    });
    read_cb_container.insert("container_process", cpro_cbs);
    

    let mut con_cbs = CallbackContainerType::new();
    con_cbs.insert("profile_full", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.connection_profile_full(id)?))
    });
    con_cbs.insert("list", |cb, _args| {
        Ok(value_to_capsule(cb.lock()?.connection_list()?))
    });
    read_cb_container.insert("connection", con_cbs);
    

    let mut ec_cbs = CallbackContainerType::new();
    ec_cbs.insert("profile_full", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.ec_profile_full(id)?))
    });
    ec_cbs.insert("list", |cb, _args| {
        Ok(value_to_capsule(cb.lock()?.ec_list()?))
    });
    ec_cbs.insert("get_state", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock()?.ec_get_state(id)?))
    });
    read_cb_container.insert("execution_context", ec_cbs);

    read_cb_container
}



pub(crate) fn update_callback_container() -> ClassCallbackContainerType {
    let mut update_cb_container = ClassCallbackContainerType::new();

    let mut proc_cbs = CallbackContainerType::new();
    proc_cbs.insert("call", |cb, args| {
        log::trace!("update_callback_container()/anonymous func() for 'process/call' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?.as_str();
        cb.lock()?.process_call(&id.to_owned(), args)
    });
    proc_cbs.insert("execute", |cb, args| {
        log::trace!("update_callback_container()/anonymous func() for 'process/execute' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        cb.lock()?.process_execute(id)
    });
    update_cb_container.insert("process", proc_cbs);


    let mut cont_proc_cbs = CallbackContainerType::new();
    cont_proc_cbs.insert("call", |cb, args| {
        log::trace!("update_callback_container()/anonymous func() for 'container_process/call' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?.as_str();
        cb.lock()?.container_process_call(&id.to_owned(), args)
    });
    cont_proc_cbs.insert("execute", |cb, args| {
        log::trace!("update_callback_container()/anonymous func() for 'container_process/execute' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        cb.lock()?.container_process_execute(id)
    });
    update_cb_container.insert("container_process", cont_proc_cbs);

    let mut ec_cbs = CallbackContainerType::new();
    ec_cbs.insert("start", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock_mut()?.ec_start(id)?))
    });
    ec_cbs.insert("stop", |cb, args| {
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?;
        Ok(value_to_capsule(cb.lock_mut()?.ec_stop(id)?))
    });
    update_cb_container.insert("execution_context", ec_cbs);

    update_cb_container
}


pub(crate) fn delete_callback_container() -> ClassCallbackContainerType {
    let mut delete_cb_container = ClassCallbackContainerType::new();

    let mut proc_cbs = CallbackContainerType::new();
    proc_cbs.insert("destroy", |cb, args| {
        log::trace!("delete_callback_container()/anonymous func() for 'process/destroy' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?.as_str();
        let v = cb.lock_mut()?.process_destroy(&id.to_owned())?;
        Ok(v.into())
    });
    delete_cb_container.insert("process", proc_cbs);

    let mut cont_proc_cbs = CallbackContainerType::new();
    cont_proc_cbs.insert("destroy", |cb, args| {
        log::trace!("delete_callback_container()/anonymous func() for 'container_process/destroy' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?.as_str();
        let v = cb.lock_mut()?.container_process_destroy(&id.to_owned())?;
        Ok(v.into())
    });
    delete_cb_container.insert("container_process", cont_proc_cbs);

    let mut cont_cbs = CallbackContainerType::new();
    cont_cbs.insert("destroy", |cb, args| {
        log::trace!("delete_callback_container()/anonymous func() for 'container/destroy' called");
        let id = args.get_param("identifier").ok_or_else(||{anyhow::Error::from(JuizError::CRUDBrokerCanNotParameterFunctionError { key_name: "identifier".to_owned() })})?.as_str();
        let v = cb.lock_mut()?.container_destroy(&id.to_owned())?;
        Ok(v.into())
    });
    delete_cb_container.insert("container", cont_cbs);

    delete_cb_container
}