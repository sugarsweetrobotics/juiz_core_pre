use juiz_core::ecs::execution_context_core::ExecutionContextState;
use juiz_core::value::{obj_get_f64, obj_get_str};
use juiz_core::{jvalue, System};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use juiz_core::{JuizResult, Value};
use juiz_core::utils::juiz_lock;

use juiz_core::ecs::{ExecutionContext, ExecutionContextCore, ExecutionContextFactory};

pub struct MainLoopEC {
    name: String,
    rate: f64,
    //core: Option<Arc<Mutex<ExecutionContextCore>>>,
}

impl MainLoopEC {
    pub fn new(name: &str, rate: f64) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self{
            name: name.to_owned(),
            rate,
            //core: None
        }))
    }
}

impl ExecutionContext for MainLoopEC {
    

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn type_name(&self) -> &str {
        "MainLoopEC"
    }

    fn profile(&self) -> JuizResult<Value> {
        Ok(jvalue!({
            "rate": self.rate,
        }))
    }

    fn execute(&self, core: &Arc<Mutex<ExecutionContextCore>>) -> JuizResult<bool> {
        juiz_lock(&core)?.svc().and(Ok(false))
    }

    fn on_load(&mut self, system: &mut System, core: Arc<Mutex<ExecutionContextCore>>) -> () {
        log::trace!("MainLoopEC({:}).on_load() called", self.name);
        let c = core.clone();
        let func: Box<dyn Fn() -> JuizResult<()>> = Box::new(move || -> JuizResult<()> {
            
            match juiz_lock(&c) {
                Err(e) => return Err(e),
                Ok(cc) => {
                    match cc.get_state() {
                        ExecutionContextState::STARTED => cc.svc().and(Ok(())),
                        _ => Ok(())
                    }
                }
            }
        });
        system.set_spin_callback(func);
        system.set_spin_sleeptime(Duration::from_micros((1000000.0 / self.rate) as u64));
    }

    fn is_periodic(&self) -> bool {
        return false;
    }
}


struct MainLoopECFactory {

}

impl ExecutionContextFactory for MainLoopECFactory {
    fn type_name(&self) -> &str {
        "MainLoopEC"
    }

    fn create(&self, manifest: Value) -> JuizResult<Arc<RwLock<dyn ExecutionContext>>> {
        let name = obj_get_str(&manifest, "name")?;
        let rate = obj_get_f64(&manifest, "rate")?;
        Ok(MainLoopEC::new(name, rate)
        )
    }
}

#[no_mangle]
pub unsafe extern "Rust" fn execution_context_factory() -> JuizResult<Arc<Mutex<dyn ExecutionContextFactory>>> {
    env_logger::init();
    Ok(Arc::new(Mutex::new(MainLoopECFactory{})))
}
