
use std::path::Path;

use juiz_core::{containers::container_lock, JuizResult, System, Value};


use clap::Subcommand;


#[derive(Debug, Subcommand)]
pub(crate) enum ContSubCommands {
    /// get logs
    #[clap(arg_required_else_help = false)]
    List {
        #[arg(short = 's', default_value = "localhost:8080", help = "Host of server (ex., localhost:8080)")]
        server: String,
    },

    /// get logs
    #[clap(arg_required_else_help = false)]
    Info {
        #[arg(help = "ID of container")]
        identifier: String
    },
}

pub(crate) fn on_container(manifest: Value, working_dir: &Path, subcommand: ContSubCommands) -> JuizResult<()> {
    match on_container_inner(manifest, working_dir, subcommand) {
        Ok(_) => return Ok(()),
        Err(e) => println!("Error: {e:?}")
    };
    Ok(())
}

pub(crate) fn on_container_inner(manifest: Value, working_dir: &Path, subcommand: ContSubCommands) -> JuizResult<()> {
    match subcommand {
        ContSubCommands::List { server } => {
            System::new(manifest)?
            .set_working_dir(working_dir)
            .run_and_do_once( |system| { on_container_list(system, server) }) 
        },
        ContSubCommands::Info { identifier } => {
            System::new(manifest)?
            .set_working_dir(working_dir)
            .run_and_do_once( |system| { 
                on_container_info(system, identifier)
            }) 
        } 
    }
}

fn on_container_list(system: &mut System, _server: String) -> JuizResult<()> {
    let proc_manifests: Vec<Value> = system.container_list()?;
    let mut ids: Vec<String> = Vec::new();
    for v in proc_manifests.iter() {
        ids.push(v.as_str().unwrap().to_owned());
    }
    println!("{ids:?}");
    Ok(())
}

fn on_container_info(system: &mut System, id: String) -> JuizResult<()> {
    let p = system.container_from_id(&id);
    match p {
        Ok(ps) => println!("{:}", container_lock(&ps)?.profile_full()?),
        Err(e) => println!("Error: {e:?}"),
    }
    Ok(())
}