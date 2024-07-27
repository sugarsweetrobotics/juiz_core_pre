use std::{path::PathBuf, rc::Rc};

use anyhow::Context;

use crate::prelude::*;
use crate::{containers::{ContainerFactoryPtr, ContainerProcessFactoryPtr}, prelude::ProcessFactoryPtr, value::obj_get_str, JuizResult, Value};

use super::{cpp_plugin::CppPlugin, python_plugin::PythonPlugin, RustPlugin};


#[derive(Clone)]
pub enum JuizObjectPlugin {
    Rust(Rc<RustPlugin>),
    Python(Rc<PythonPlugin>),
    Cpp(Rc<CppPlugin>),
}


/// 引数vからpathメンバの値を引き出し、nameと連結したPathを作成する
pub fn concat_dirname(v: &serde_json::Value, name: String) -> JuizResult<PathBuf> {
    Ok(PathBuf::from(obj_get_str(v, "path")?.to_string()).join(name))
}

#[cfg(target_os = "macos")]
pub fn plugin_name_to_file_name(name: &str) -> String {
    "lib".to_owned() + name + ".dylib"
}

#[cfg(target_os = "windows")]
pub fn plugin_name_to_file_name(name: &str) -> String {
    name.to_owned() + ".dll"
}

fn plugin_name_to_python_file_name(name: &str) -> String {
    name.to_owned() + ".py"
}


/// まずnameからpluginのファイル名に変換する。macだと.dylibをつける作業。そしてvの中のpathと連結させてpathを作る
fn plugin_path(name: &str, v: &Value) -> JuizResult<std::path::PathBuf> {
    concat_dirname(v, plugin_name_to_file_name(name))
}

/// まずnameからpluginのファイル名に変換する。macだと.dylibをつける作業。そしてvの中のpathと連結させてpathを作る
fn python_plugin_path(name: &str, v: &Value) -> JuizResult<std::path::PathBuf> {
    concat_dirname(v, plugin_name_to_python_file_name(name))
}

/// まずnameからpluginのファイル名に変換する。macだと.dylibをつける作業。そしてvの中のpathと連結させてpathを作る
fn cpp_plugin_path(name: &str, v: &Value) -> JuizResult<std::path::PathBuf> {
    concat_dirname(v, plugin_name_to_file_name(name))
}


impl JuizObjectPlugin {

    pub fn  new(language: &str, name: &str, v: &Value, manifest_entry_point: &str) -> JuizResult<JuizObjectPlugin> {
        //let manifest_entry_point = "manifest_entry_point";
        match language {
            "rust" => Ok(JuizObjectPlugin::Rust(Rc::new(RustPlugin::load(plugin_path(name, v)?)?))),
            "python" => Ok( JuizObjectPlugin::Python(Rc::new(PythonPlugin::load(python_plugin_path(name, v)?)?))),
            "c++" => Ok(JuizObjectPlugin::Cpp(Rc::new(CppPlugin::new(cpp_plugin_path(name, v)?, manifest_entry_point)?))),
            _ => {
                log::error!("In setup_container_factories() function, unknown language option ({:}) detected", language);
                Err(anyhow::Error::from(JuizError::InvalidSettingError{message: format!("In setup_container_factories() function, unknown language option ({:}) detected", language)}))
            } 
        }
    }

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
                p.load_process_factory(working_dir, symbol_name)
            },
            JuizObjectPlugin::Python(p) => {
                p.load_process_factory(working_dir, symbol_name)
            },
            JuizObjectPlugin::Cpp(p) => {
                p.load_process_factory(working_dir, symbol_name)
            },
        }
    }

    pub fn load_container_factory(&self, working_dir: Option<PathBuf>, symbol_name: &str, container_profile: &Value) -> JuizResult<ContainerFactoryPtr> {
        log::trace!("load_container_factory({working_dir:?}, {symbol_name}, {container_profile}) called");
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
                p.load_container_factory(working_dir, "container_factory")
                //Ok(Arc::new(Mutex::new(CppContainerFactoryImpl::new_with_manifest(p.clone(), container_profile)?)))
            },
        }
    }

    pub fn load_container_process_factory(&self, working_dir: Option<PathBuf>, symbol_name: &str, _manifest: &Value) -> JuizResult<ContainerProcessFactoryPtr> {
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
                p.load_container_process_factory(working_dir, symbol_name)
                //Ok(Arc::new(Mutex::new(CppContainerProcessFactoryImpl::new_with_manifest(p.clone(), symbol_name, manifest)?)))
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