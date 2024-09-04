use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageToController {
    IpcSender(ipc_channel::ipc::IpcSender<IpcMessageToVersoview>),
    Message(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageToVersoview {
    Message(String),
}
