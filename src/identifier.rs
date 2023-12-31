use crate::{Value, JuizResult, value::obj_get_str};

pub type Identifier = String;



pub fn identifier_new(broker_type_name: &str, broker_name: &str, class_name: &str, type_name: &str, object_name: &str) -> Identifier {
    // "core://core/process/inc0::increment_function"
    // "http://localhost:3000/procss/inc0::increment_function"
    // "{broker_type_name}://{broker_name}/{class_name}/{object_name}::{type_name}
    broker_type_name.to_string() + "://" + broker_name + "/" + class_name + "/" + object_name + "::" + type_name
}

pub fn connection_identifier_new(source_id: Identifier, destination_id: Identifier, arg_name: &str) -> Identifier {
    "connnection://".to_string() + source_id.as_str() + "|" + arg_name + "|" + destination_id.as_str()
}

pub fn identifier_from_manifest(default_broker_type_name: &str, default_broker_name: &str, class_name: &str, manifest: &Value) -> JuizResult<Identifier> {
    match obj_get_str(manifest, "identifier") {
        Ok(id) => Ok(id.to_string()), 
        Err(_) => {
            let object_name = obj_get_str(manifest, "name")?;
            let type_name = obj_get_str(manifest, "type_name")?;

            let broker_name = obj_get_str(manifest, "broker_name").or::<anyhow::Error>(Ok(default_broker_name))?;
            let broker_type_name = obj_get_str(manifest, "broker_type_name").or::<anyhow::Error>(Ok(default_broker_type_name))?;
            Ok(identifier_new(broker_type_name, broker_name, class_name, type_name, object_name))
        }
    }
}

pub(crate) fn _create_identifier(class_name: &str, type_name: &str, object_name: &str) -> Identifier {
    identifier_new("core", "core", class_name, type_name, object_name)
}

pub(crate) fn _create_factory_identifier(class_name: &str, type_name: &str) -> Identifier {
    identifier_new("core", "core", class_name, type_name, type_name)
}

pub(crate) fn create_identifier_from_manifest(class_name: &str, manifest: &Value) -> JuizResult<Identifier> {
    identifier_from_manifest("core", "core", class_name, manifest)
}


#[derive(PartialEq, Debug)]
pub struct IdentifierStruct {
    pub identifier: Identifier,
    pub class_name: String, 
    pub type_name: String,
    pub object_name: String,
    pub broker_name: String,
    pub broker_type_name: String,
}

impl From<Identifier> for IdentifierStruct {
    fn from(identifier: Identifier) -> Self {
        digest_identifier(&identifier)
    }
}
impl IdentifierStruct {
    
    pub fn to_identifier(&self) -> Identifier {
        return identifier_new(self.broker_type_name.as_str(), 
                self.broker_name.as_str(), 
                self.class_name.as_str(), 
                self.type_name.as_str(), 
                self.object_name.as_str())
    }

    pub fn set_class_name<'a>(&'a mut self, class_name: &str) -> &'a IdentifierStruct{
        self.class_name = class_name.to_string();
        self
    }
}


///
///
fn digest_identifier(identifier: &Identifier) -> IdentifierStruct {
    let re = regex::Regex::new(r"^(.+?)://(.+?)/(.+?)/(.+?)::(.+?)$").unwrap();
    let caps = re.captures(identifier).unwrap();
    let class_name = caps[3].to_owned();
    let type_name = caps[5].to_owned();
    let object_name = caps[4].to_owned();
    let broker_name = caps[2].to_owned();
    let broker_type_name = caps[1].to_owned();
    IdentifierStruct{ 
        identifier: identifier.clone(), 
        class_name, 
        type_name, 
        object_name, 
        broker_name, 
        broker_type_name}
}
