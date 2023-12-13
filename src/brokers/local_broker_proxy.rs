use std::{sync::{Arc, Mutex}, time::Duration, ops::Deref};

use crate::{jvalue, JuizResult, Value, JuizError, brokers::messenger_broker_proxy_factory::{MessengerBrokerProxyFactory, create_messenger_broker_proxy_factory}, object::{ObjectCore, JuizObjectClass}, BrokerProxyFactory};

use super::{local_broker::SenderReceiverPair, messenger_broker_proxy::{MessengerBrokerProxy, MessengerBrokerProxyCore, SendReceivePair, MessengerBrokerProxyCoreFactory}};



pub type LocalBrokerProxy = MessengerBrokerProxy;
pub struct LocalBrokerProxyCore {
    sender_receiver: Arc<Mutex<SenderReceiverPair>>,
}

pub struct LocalBrokerProxyCoreFactory {
    sender_receiver: Arc<Mutex<SenderReceiverPair>>,
}

impl LocalBrokerProxyCoreFactory {
    pub fn new(sender_receiver: Arc<Mutex<SenderReceiverPair>>) -> JuizResult<Box<dyn MessengerBrokerProxyCoreFactory>> {
        Ok(Box::new(LocalBrokerProxyCoreFactory {sender_receiver}))
    }
}

impl MessengerBrokerProxyCoreFactory  for LocalBrokerProxyCoreFactory {
    fn create_core(&self, object_name: &str) -> JuizResult<Box<dyn MessengerBrokerProxyCore>> {
        Ok(Box::new(LocalBrokerProxyCore{sender_receiver: self.sender_receiver.clone()}))
    }
}

impl MessengerBrokerProxyCore for LocalBrokerProxyCore {
    fn send_and_receive(&self, value: Value, timeout: Duration) -> JuizResult<Value> {
        let sndr_recvr = self.sender_receiver.lock().map_err(|_e| return anyhow::Error::from(JuizError::BrokerSendCanNotLockSenderError{}))?;
        let SenderReceiverPair(sndr, recvr) = sndr_recvr.deref();
        let _ = sndr.send(value).map_err(|e| return anyhow::Error::from(JuizError::LocalBrokerProxySendError{send_error: e}))?;
        recvr.recv_timeout(timeout).map_err(|e|
                return anyhow::Error::from(JuizError::LocalBrokerProxyReceiveTimeoutError{error: e}))
    }
}

impl LocalBrokerProxyCore {
    pub fn new(sender_receiver: Arc<Mutex<SenderReceiverPair>> ) -> LocalBrokerProxyCore {
        LocalBrokerProxyCore{sender_receiver}
    }
}

pub fn create_local_broker_proxy_factory(sender_receiver: Arc<Mutex<SenderReceiverPair>>) -> JuizResult<Arc<Mutex<dyn BrokerProxyFactory>>> {
    log::trace!("create_local_broker_factory called");
    create_messenger_broker_proxy_factory("LocalBrokerProxyFactory", "local", LocalBrokerProxyCoreFactory::new(sender_receiver)?)
}
