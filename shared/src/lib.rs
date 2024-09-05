use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageToController {
    IpcSender(ipc_channel::ipc::IpcSender<IpcMessageToVersoview>),
    Echo(String, ipc_channel::ipc::IpcSender<String>),
    Message(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageToVersoview {
    Message(String),
    NavigateTo(String),
}
