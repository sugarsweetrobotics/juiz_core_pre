
use crate::{Value, Identifier, JuizResult, JuizObject};

use super::connection::Connection;

pub trait DestinationConnection : Connection {

    fn execute_destination(&self) -> JuizResult<Value>;

    fn push(&self, value: &Value) -> JuizResult<Value>;

}