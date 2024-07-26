use std::{path::PathBuf, rc::Rc, sync::{Arc, Mutex}};

use anyhow::Context;

use crate::{containers::{cpp_container_factory_impl::CppContainerFactoryImpl, cpp_container_process_factory_impl::CppContainerProcessFactoryImpl, ContainerFactoryPtr, ContainerProcessFactoryPtr}, prelude::ProcessFactoryPtr, processes::cpp_process_factory_impl::CppProcessFactoryImpl, JuizResult, Value};

use super::{cpp_plugin::CppPlugin, python_plugin::PythonPlugin, RustPlugin};


#[allow(unused)]
#[derive(Clone)]
pub enum JuizObjectPlugin {
    Rust(Rc<RustPlugin>),
    Python(Rc<PythonPlugin>),
    Cpp(Rc<CppPlugin>),
}

impl JuizObjectPlugin {

    pub fn profile_full(&self) -> JuizResult<Value> {
        match self {
            JuizObjectPlugin::Rust(p) => p.profile_full(),
            JuizObjectPlugin::Python(p) => p.profile_full(),
            JuizObjectPlugin::Cpp(p) => p.profile_full(),
        }
    }

    pub fn load_process_factory(&self, working_dir: Option<PathBuf>, symbol_name: &str) -> JuizResult<ProcessFactoryPtr> {
        match self {
            JuizObjectPlugin::Rust(p) => {
                type SymbolType = libloading::Symbol<'static, unsafe extern "Rust" fn() -> JuizResult<ProcessFactoryPtr>>;
                unsafe {
                    let symbol = p.load_symbol::<SymbolType>(symbol_name.as_bytes())?;
                    (symbol)().with_context(||format!("calling symbol '{symbol_name}'"))
                }
            },
            JuizObjectPlugin::Python(p) => {
                p.load_process_factory(working_dir, symbol_name)
            },
            JuizObjectPlugin::Cpp(p) => {
                Ok(Arc::new(Mutex::new(CppProcessFactoryImpl::new(p.clone())?)))
            },
        }
    }

    pub fn load_container_factory(&self, working_dir: Option<PathBuf>, symbol_name: &str, container_profile: &Value) -> JuizResult<ContainerFactoryPtr> {
        match self {
            JuizObjectPlugin::Rust(p) => {
                type SymbolType = libloading::Symbol<'static, unsafe extern "Rust" fn() -> JuizResult<ContainerFactoryPtr>>;
                unsafe {
                    let symbol = p.load_symbol::<SymbolType>(symbol_name.as_bytes())?;
                    (symbol)().with_context(||format!("calling symbol '{symbol_name}'"))
                }
            },
            JuizObjectPlugin::Python(p) => {
                p.load_container_factory(working_dir, "container_factory")
            },
            JuizObjectPlugin::Cpp(p) => {
                Ok(Arc::new(Mutex::new(CppContainerFactoryImpl::new_with_manifest(p.clone(), container_profile)?)))
            },
        }
    }

    pub fn load_container_process_factory(&self, working_dir: Option<PathBuf>, symbol_name: &str, manifest: &Value) -> JuizResult<ContainerProcessFactoryPtr> {
        log::trace!("load_container_process_factory({working_dir:?}, {symbol_name}) called");
        match self {
            JuizObjectPlugin::Rust(p) => {
                type SymbolType = libloading::Symbol<'static, unsafe extern "Rust" fn() -> JuizResult<ContainerProcessFactoryPtr>>;
                unsafe {
                    let symbol = p.load_symbol::<SymbolType>(symbol_name.as_bytes())?;
                    (symbol)().with_context(||format!("calling symbol '{symbol_name}'"))
                }
            },
            JuizObjectPlugin::Python(p) => {
                p.load_container_process_factory(working_dir, symbol_name)
            },
            JuizObjectPlugin::Cpp(p) => {
                Ok(Arc::new(Mutex::new(CppContainerProcessFactoryImpl::new_with_manifest(p.clone(), symbol_name, manifest)?)))
            },
        }
    }

    pub fn load_component_profile(&self, working_dir: Option<PathBuf>) -> JuizResult<Value> {
        match self {
            JuizObjectPlugin::Rust(p) => p.load_component_profile(),
            JuizObjectPlugin::Python(p) => p.load_component_profile(working_dir),
            JuizObjectPlugin::Cpp(p) => p.load_component_profile(working_dir),
        }
    }
}

pub trait Plugin {
    fn profile_full(&self) -> JuizResult<Value>;
}