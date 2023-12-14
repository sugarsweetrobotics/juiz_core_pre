use crate::jvalue;
use crate::{JuizObject, JuizError, JuizResult, object::{ObjectCore, JuizObjectClass}, Value, Identifier, utils::check_connection_manifest, value::obj_get_str};



pub enum ConnectionType {
    Pull,
    Push
}

impl ToString for ConnectionType {

    fn to_string(&self) -> String {
        match self {
            ConnectionType::Pull => "Pull".to_string(),
            ConnectionType::Push => "Push".to_string()
        }
    }
}

pub fn connection_type_from(typ_str_result: JuizResult<&str>) -> JuizResult<&'static ConnectionType> {
    if typ_str_result.is_err() {
        return Ok(&ConnectionType::Pull);
    }
    let typ_str = typ_str_result.unwrap();
    match typ_str {
        "pull" => Ok(&ConnectionType::Pull),
        "push" => Ok(&ConnectionType::Push),
        _ => {
            Err(anyhow::Error::from(JuizError::ConnectionTypeError { type_string: typ_str.to_string() }))
        }
    }
}


pub struct ConnectionCore {
    core: ObjectCore, 
    manifest: Value,
    connection_type: &'static ConnectionType,
    source_process_identifier: Identifier, 
    destination_process_identifier: Identifier,
    arg_name: String,
}

impl Clone for ConnectionCore {
    fn clone(&self) -> Self {
        Self { core: self.core.clone(),
             manifest: self.manifest.clone(), 
             connection_type: self.connection_type, 
             source_process_identifier: self.source_process_identifier.clone(), 
             destination_process_identifier: self.destination_process_identifier.clone(), 
             arg_name: self.arg_name.clone() }
    }
}

impl ConnectionCore { 

    pub fn new(connection_impl_class_name: &'static str, source_process_identifier: Identifier, destination_process_identifier: Identifier, arg_name: String, connection_manifest: &Value) -> JuizResult<Self> {
        log::trace!("ConnectionCore::new() called");
        let manif = check_connection_manifest(connection_manifest.clone())?;
        let connection_type = connection_type_from(obj_get_str(&manif, "type"))?;
        let connection_id = obj_get_str(&manif, "id")?;
        Ok(ConnectionCore {
            core: ObjectCore::create(JuizObjectClass::Connection(connection_impl_class_name), connection_impl_class_name, connection_id),
            source_process_identifier,
            destination_process_identifier,
            manifest: manif,
            arg_name,
            connection_type})
    }

    pub fn object_core(&self) -> &ObjectCore {
        &self.core
    }

    pub fn destination_identifier(&self) -> &Identifier {
        &self.destination_process_identifier
    }

    pub fn source_identifier(&self) -> &Identifier {
        &self.source_process_identifier
    }

    pub fn arg_name(&self) -> &String {
        &self.arg_name
    }

    pub fn connection_type(&self) -> &'static ConnectionType {
        &self.connection_type
    }

    pub fn profile_full(&self) -> JuizResult<Value> {
        Ok(jvalue!({
            "identifier": self.core.identifier(),
            "connection_type": self.connection_type.to_string(),
            "arg_name": self.arg_name().to_owned(),
            "destination_identifier": self.destination_identifier().to_owned(),
            "source_process_identifier": self.source_process_identifier.to_owned(),
        }))
    }
}



pub trait Connection : JuizObject {

    fn connection_core(&self) -> &ConnectionCore;

    fn arg_name(&self) -> &String {
        self.connection_core().arg_name()
    }

    fn connection_type(&self) -> &ConnectionType {
        self.connection_core().connection_type()
    }
}
