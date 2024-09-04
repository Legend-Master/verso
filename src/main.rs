// Prevent console window from appearing on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use shared::{IpcMessageToController, IpcMessageToVersoview};
use verso::config::Config;
use verso::{Result, Verso};
use winit::application::ApplicationHandler;
use winit::event_loop::{self, DeviceEvents};
use winit::event_loop::{EventLoop, EventLoopProxy};

struct App {
    verso: Option<Verso>,
    proxy: EventLoopProxy<()>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let config = Config::new(resources_dir_path().unwrap());
        self.verso = Some(Verso::new(event_loop, self.proxy.clone(), config));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(v) = self.verso.as_mut() {
            v.handle_winit_window_event(window_id, event);
            v.handle_servo_messages(event_loop);
        }
    }

    fn user_event(&mut self, event_loop: &event_loop::ActiveEventLoop, _: ()) {
        if let Some(v) = self.verso.as_mut() {
            v.handle_servo_messages(event_loop);
        }
    }
}

/// Args used in the webview mode
#[derive(Debug, argh::FromArgs)]
struct IpcServerArgs {
    /// the IPC channel id
    #[argh(option)]
    ipc_channel: Option<String>,
}

fn main() -> Result<()> {
    let server_args: IpcServerArgs = argh::from_env();
    if let Some(channel_name) = server_args.ipc_channel {
        dbg!(&channel_name);
        let sender =
            ipc_channel::ipc::IpcSender::<IpcMessageToController>::connect(channel_name).unwrap();
        let (controller_sender, receiver) =
            ipc_channel::ipc::channel::<IpcMessageToVersoview>().unwrap();
        sender
            .send(IpcMessageToController::IpcSender(controller_sender))
            .unwrap();
        sender
            .send(IpcMessageToController::Message("data".to_owned()))
            .unwrap();
        sender
            .send(IpcMessageToController::Message("more data".to_owned()))
            .unwrap();
        while let Ok(data) = receiver.try_recv_timeout(Duration::from_secs(1)) {
            std::thread::sleep(Duration::from_millis(10));
            dbg!(data);
        }
        let (echo_sender, echo_receiver) = ipc_channel::ipc::channel::<String>().unwrap();
        sender
            .send(IpcMessageToController::Echo("echo".to_owned(), echo_sender))
            .unwrap();
        dbg!(echo_receiver.recv().unwrap());
        return Ok(());
    }

    let event_loop = EventLoop::new()?;
    event_loop.listen_device_events(DeviceEvents::Never);
    let proxy = event_loop.create_proxy();
    let mut app = App { verso: None, proxy };
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn resources_dir_path() -> Option<std::path::PathBuf> {
    #[cfg(feature = "packager")]
    let root_dir = {
        use cargo_packager_resource_resolver::{current_format, resources_dir};
        current_format().and_then(|format| resources_dir(format))
    };
    #[cfg(feature = "flatpak")]
    let root_dir = {
        use std::str::FromStr;
        std::path::PathBuf::from_str("/app")
    };
    #[cfg(not(any(feature = "packager", feature = "flatpak")))]
    let root_dir = std::env::current_dir();

    root_dir.ok().map(|dir| dir.join("resources"))
}
