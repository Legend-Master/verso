use shared::{IpcMessageToController, IpcMessageToVersoview};
use std::{path::PathBuf, process::Command};

use ipc_channel::ipc::IpcOneShotServer;

pub fn init(versoview_path: PathBuf) {
    let (server, server_name) = IpcOneShotServer::<IpcMessageToController>::new().unwrap();
    Command::new(versoview_path)
        .args(["--ipc-channel", &server_name])
        .spawn()
        .unwrap();
    let (reveiver, data) = server.accept().unwrap();
    let IpcMessageToController::IpcSender(sender) = data else {
        panic!();
    };
    while let Ok(data) = reveiver.recv() {
        if let IpcMessageToController::Message(message) = data {
            dbg!(&message);
            sender
                .send(IpcMessageToVersoview::Message(message))
                .unwrap();
        }
    }
}
