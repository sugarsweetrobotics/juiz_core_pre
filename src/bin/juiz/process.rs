
use std::path::Path;
use juiz_core::utils::yaml_conf_load;
use juiz_core::value::load_str;
use juiz_core::log;

use juiz_core::prelude::*;
use juiz_core::processes::proc_lock;
use juiz_core::opencv::imgcodecs::imwrite;
use juiz_core::opencv::core::{Mat, Vector};
use clap::Subcommand;

use crate::Args;

#[derive(Debug, Subcommand, Clone)]
pub(crate) enum ProcSubCommands {

    /// get logs
    #[clap(arg_required_else_help = false)]
    List {
        #[arg(short = 'a', help = "Any process includes")]
        any_process: bool,
        
        #[arg(short = 'f', default_value = "./juiz.conf", help = "Input system definition file path")]
        filepath: String,
    },

    /// get logs
    #[clap(arg_required_else_help = false)]
    Info {
        #[arg(help = "ID of process")]
        identifier: String
    },

    /// get logs
    #[clap(arg_required_else_help = false)]
    Call {
        #[arg(help = "ID of process")]
        identifier: String,


        #[arg(help = "Argument")]
        argument: String,

        #[arg(short = 'o', help = "Output Filename")]
        fileout: Option<String>,
    },

}


pub(crate) fn on_process(manifest: Value, working_dir: &Path, subcommand: ProcSubCommands, args: Args) -> JuizResult<()> {
    match on_process_inner(manifest, working_dir, subcommand, args) {
        Ok(_) => return Ok(()),
        Err(e) => println!("Error: {e:?}")
    };
    Ok(())
}
pub(crate) fn on_process_inner(manifest: Value, working_dir: &Path, subcommand: ProcSubCommands, args: Args) -> JuizResult<()> {
    match subcommand {
        ProcSubCommands::List { any_process, filepath} => {
            log::trace!("process list command is selected. args={args:?}");
            let manifest2 = yaml_conf_load(filepath.clone())?;
            let server = args.server;
            System::new(manifest2)?
                .set_working_dir(working_dir)
                .start_http_broker(args.start_http_broker)
                .setup()?
                .add_subsystem_by_id(server.clone())?
                .run_and_do_once( |system| { 
                if any_process {
                    on_any_process_list(system, server)
                } else {
                    on_process_list(system, server)
                } 
            }) 
        },
        ProcSubCommands::Info { identifier } => {
            System::new(manifest)?
                .set_working_dir(working_dir)
                .start_http_broker(args.start_http_broker)
                .setup()?
                .run_and_do_once( |system| { 
                on_process_info(system, identifier)
            }) 
        },
        ProcSubCommands::Call { identifier, argument , fileout} => {
            System::new(manifest)?
                .set_working_dir(working_dir)
                .start_http_broker(args.start_http_broker)
                .setup()?
                .run_and_do_once( |system| { 
                on_process_call(system, identifier, argument, fileout)
            }) 
        } 
    }
}

fn on_process_list(system: &mut System, _server: String) -> JuizResult<()> {
    log::info!("on_process_list() called");
    let proc_manifests: Vec<Value> = system.process_list()?;
    let mut ids: Vec<String> = Vec::new();
    for v in proc_manifests.iter() {
        ids.push(v.as_str().unwrap().to_owned());
    }
    println!("{ids:?}");
    Ok(())
}

fn on_any_process_list(system: &mut System, _server: String) -> JuizResult<()> {
    let proc_manifests: Vec<Value> = system.any_process_list()?;
    let mut ids: Vec<String> = Vec::new();
    for v in proc_manifests.iter() {
        ids.push(v.as_str().unwrap().to_owned());
    }
    println!("{ids:?}");
    Ok(())
}

fn on_process_info(system: &mut System, id: String) -> JuizResult<()> {
    //println!("processes:");
    let p = system.any_process_from_id(&id);
    match p {
        Ok(ps) => println!("{:}", proc_lock(&ps)?.profile_full()?),
        Err(e) => println!("Error: {e:?}"),
    }
    Ok(())
}

fn on_process_call(system: &mut System, id: String, arg: String, fileout: Option<String>) -> JuizResult<()> {
    //println!("processes:");
    let p = system.any_process_from_id(&id);
    match p {
        Ok(ps) => {
            let argv = load_str(arg.as_str())?;
            // println!("Value is {argv:?}");
            let value = proc_lock(&ps)?.call(argv.try_into()?)?;
            if value.is_value()? {
                println!("{:?}", value);
            } else if value.is_mat()? {
                let _ = value.lock_as_mat(|mat: &Mat| {

                    let params: Vector<i32> = Vector::new();
                    match fileout {
                        Some(filepath) => {
                            match imwrite(filepath.as_str(), mat, &params) {
                                Ok(_) => {
                                    //println!("ok");
                                },
                                Err(e) => {
                                    println!("error: {e:?}");
                                }
                            }
                        },
                        None => {
                            println!("{:?}", mat);
                        }
                    }
                } );
            }
        },
        Err(e) => println!("Error: {e:?}"),
    }
    Ok(())
}