
use juiz_core::prelude::*;

fn main() -> JuizResult<()>{

    let manifest = jvalue!(
        {

            "name": "test_system",
            "plugins": {
                "process_factories": {
                    "increment_process": {
                        "path": "./target/debug"
                    }
                },
                "broker_factories": {
                    "http_broker": {
                        "path": "./target/debug"
                    }
                }
            },
            "brokers": [
                {
                    "type_name": "http",
                    "name": "localhost_8000",
                    "host": "localhost",
                    "port": 8000,
                }  
            ],
            "processes": [
                {
                    "type_name": "increment_process",
                    "name": "increment0"
                },
            ]
        }
    );

    Ok(System::new(manifest)?.run_and_do(|system|{
        println!("JuizSystem started!!");
        let v = system.broker_proxy(&jvalue!({"type_name":"local"}))?.lock().unwrap().system_profile_full()?;
        println!("System: {:#}", v);
        Ok(())
    }).expect("Error in System::run_and_do()"))
}