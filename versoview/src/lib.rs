use shared::{IpcMessageToController, IpcMessageToVersoview};
use std::{path::PathBuf, process::Command, thread::sleep, time::Duration};

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

    let cloned_sender = sender.clone();
    std::thread::spawn(move || {
        sleep(Duration::from_secs(5));
        cloned_sender
            .send(IpcMessageToVersoview::NavigateTo(
                "https://google.com".to_owned(),
            ))
            .unwrap();
    });

    while let Ok(data) = reveiver.recv() {
        match data {
            IpcMessageToController::Echo(value, sender) => sender.send(value).unwrap(),
            IpcMessageToController::Message(message) => {
                dbg!(&message);
                sender
                    .send(IpcMessageToVersoview::Message(message))
                    .unwrap();
            }
            _ => {}
        };
    }
}
