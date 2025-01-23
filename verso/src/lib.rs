use log::error;
use std::{
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};
use versoview_messages::ControllerMessage;

use ipc_channel::{
    ipc::{self, IpcOneShotServer, IpcSender},
    router::ROUTER,
};

#[derive(Default)]
struct EventListeners {
    on_navigation_starting: Arc<Mutex<Option<Box<dyn Fn(url::Url) -> bool + Send + 'static>>>>,
}

#[derive(Debug, Default)]
pub struct VersoviewSettings {
    pub with_panel: bool,
}

pub struct VersoviewController {
    sender: IpcSender<ControllerMessage>,
    event_listeners: EventListeners,
}

impl VersoviewController {
    /// Create a new verso instance with settings and get the controller to it
    fn create(
        verso_path: impl AsRef<Path>,
        initial_url: url::Url,
        settings: VersoviewSettings,
    ) -> Self {
        let path = verso_path.as_ref();
        let (server, server_name) =
            IpcOneShotServer::<IpcSender<ControllerMessage>>::new().unwrap();
        let mut command = Command::new(path);
        command
            .arg(format!("--ipc-channel={server_name}"))
            .arg(format!("--url={initial_url}"));
        if !settings.with_panel {
            command.arg("--no-panel");
        }
        command.spawn().unwrap();
        let (_, sender) = server.accept().unwrap();
        Self {
            sender,
            event_listeners: EventListeners::default(),
        }
    }

    /// Create a new verso instance with default settings and get the controller to it
    pub fn new(verso_path: impl AsRef<Path>, initial_url: url::Url) -> Self {
        Self::create(verso_path, initial_url, VersoviewSettings::default())
    }

    /// Create a new verso instance with custom settings and get the controller to it
    pub fn new_with_settings(
        verso_path: impl AsRef<Path>,
        initial_url: url::Url,
        settings: VersoviewSettings,
    ) -> Self {
        Self::create(verso_path, initial_url, settings)
    }

    /// Navigate to url
    pub fn navigate(&self, url: url::Url) -> Result<(), Box<ipc_channel::ErrorKind>> {
        self.sender.send(ControllerMessage::NavigateTo(url))
    }

    /// Listen on navigation starting triggered by user click on a link,
    /// return a boolean in the callback to decide whether or not allowing this navigation
    pub fn on_navigation_starting(
        &self,
        callback: impl Fn(url::Url) -> bool + Send + 'static,
    ) -> Result<(), Box<ipc_channel::ErrorKind>> {
        if self
            .event_listeners
            .on_navigation_starting
            .lock()
            .unwrap()
            .replace(Box::new(callback))
            .is_some()
        {
            return Ok(());
        }
        let cb = self.event_listeners.on_navigation_starting.clone();
        let (sender, receiver) = ipc::channel::<(url::Url, ipc::IpcSender<bool>)>()?;
        self.sender
            .send(ControllerMessage::OnNavigationStarting(sender))?;
        ROUTER.add_typed_route(
            receiver,
            Box::new(move |message| match message {
                Ok((url, result_sender)) => {
                    if let Err(e) = result_sender.send(cb.lock().unwrap().as_ref().unwrap()(url)) {
                        error!("Error while sending back OnNavigationStarting result: {e}");
                    }
                }
                Err(e) => error!("Error while receiving OnNavigationStarting message: {e}"),
            }),
        );
        Ok(())
    }
}
