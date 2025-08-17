use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use thiserror::Error;

pub static LOCAL_MAP: once_cell::sync::Lazy<
    Arc<DashMap<Component, tokio::sync::mpsc::Sender<Vec<u8>>>>,
> = Lazy::new(|| Arc::new(DashMap::new()));

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Component {
    pub component_type: ComponentType,
    pub component_name: String,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentType {
    Apiserver,
    Proxy,
    Gameserver,
}

#[allow(dead_code)]
pub fn register_service(
    component: Component,
    sender: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    LOCAL_MAP.insert(component, sender);
    Ok(())
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Channel broken: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),
}

pub async fn send_message_no_name(
    component_type: ComponentType,
    data: Vec<u8>,
) -> Result<(), RegistryError> {
    let tx = LOCAL_MAP
        .iter()
        .find(|x| x.key().component_type == component_type)
        .map(|x| x.value().clone());

    if let Some(tx) = tx {
        tx.send(data).await?;
    }
    Ok(())
}
