use std::sync::{Mutex, Arc};



use crate::{JuizResult, Value};

use super::execution_context_core::ExecutionContextCore;

pub trait ECServiceFunction : Fn()->JuizResult<()> + Send + Sync {

}

pub trait ExecutionContext {

    fn name(&self) -> &str;

    fn type_name(&self) -> &str;

    fn on_starting(&mut self, _svc: Arc<Mutex<ExecutionContextCore>>) -> JuizResult<()> {
        Ok(())
    }

    fn on_stopping(&mut self, _core: Arc<Mutex<ExecutionContextCore>>) -> JuizResult<()> {
        Ok(())
    }

    fn profile(&self) -> JuizResult<Value>;
}