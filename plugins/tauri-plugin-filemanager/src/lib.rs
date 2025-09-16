use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Filemanager;
#[cfg(mobile)]
use mobile::Filemanager;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the filemanager APIs.
pub trait FilemanagerExt<R: Runtime> {
    fn filemanager(&self) -> &Filemanager<R>;
}

impl<R: Runtime, T: Manager<R>> crate::FilemanagerExt<R> for T {
    fn filemanager(&self) -> &Filemanager<R> {
        self.state::<Filemanager<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("filemanager")
        .invoke_handler(tauri::generate_handler![commands::ping])
        .setup(|app, api| {
            #[cfg(mobile)]
            let filemanager = mobile::init(app, api)?;
            #[cfg(desktop)]
            let filemanager = desktop::init(app, api)?;
            app.manage(filemanager);
            Ok(())
        })
        .build()
}
