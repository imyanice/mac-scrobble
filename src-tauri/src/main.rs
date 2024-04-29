mod listener;

use rustfm_scrobble::{Scrobbler, ScrobblerError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{thread, time};
use tauri::api::notification::Notification;
use tauri::api::process::restart;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowUrl,
};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub setup: bool,
    pub enabled: bool,
}

fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save_credentials])
        .system_tray(get_system_tray())
        .on_system_tray_event(handle_tray_click)
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    let app_handle = app.handle();
    if get_config().setup {
        thread::spawn(move || {
            listener::listen(app_handle);
        });
    } else {
        let mut config = get_config();
        config.enabled = true;
        set_config(config);
        tauri::WindowBuilder::new(&app, "local", WindowUrl::App("index.html".into()))
            .title("Last.FM Login")
            .center()
            .build()
            .expect("could not build window");
    }

    app.run(move |_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if get_config().setup {
                api.prevent_exit();
            }
        }
    });
}

pub fn get_config() -> Config {
    let path = data_dir().join("config.toml");

    confy::load_path(path).expect("Could not load config")
}

pub fn set_config(config: Config) {
    let path = data_dir().join("config.toml");

    confy::store_path(path, config).expect("Could not save config");
}

pub fn data_dir() -> PathBuf {
    let home_dir = std::env::var_os("HOME").map(PathBuf::from).unwrap();
    home_dir
        .join("Library/Application Support")
        .join("me.yanice.mac-scrobble")
}

#[tauri::command(rename_all = "snake_case")]
fn save_credentials(app_handle: AppHandle, username: String, password: String) {

    let mut scrobbler = Scrobbler::new(api_key(), api_secret());
    let auth = scrobbler.authenticate_with_password(username.as_str(), password.as_str());
    match auth {
        Ok(_) => {
            let mut config = get_config();
            config.username = username;
            config.password = password;
            if !config.setup {
                config.setup = true;
            }
            set_config(config);
            Notification::new(app_handle.config().tauri.bundle.identifier.clone())
                .title("Last.FM Scrobbler")
                .body("You are now logged in. The scrobbler will now live in your menubar.")
                .show()
                .expect("couldnt shouw notification");
            thread::sleep(time::Duration::from_millis(1000));
            restart(&app_handle.env());
        }
        Err(err) => {
            println!("{}", err);
            Notification::new(app_handle.config().tauri.bundle.identifier.clone())
                .title("Last.FM Scrobbler")
                .body("Invalid Last.FM credentials. Please check them.")
                .show()
                .expect("couldnt show notification");
        }
    }
}

fn get_system_tray() -> SystemTray {
    let title = CustomMenuItem::new("title", "Last.FM Scrobbler").disabled();

    let options = CustomMenuItem::new("options".to_string(), "Open settings");
    let enabled = CustomMenuItem::new("enabled", "Enabled ✅");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(title)
        .add_item(enabled)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(options)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

fn handle_tray_click(app: &tauri::AppHandle, event: SystemTrayEvent) {
    if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        let item_handle = app.tray_handle().get_item(&id);
        match id.as_str() {
            "options" => {
                if Manager::windows(app).is_empty() {
                    tauri::WindowBuilder::new(
                        app,
                        "local",
                        tauri::WindowUrl::App("index.html".into()),
                    )
                    .center()
                    .title("Last.FM Login")
                    .build()
                    .expect("error while creating local window");
                } else {
                    let main_window = app.get_window("local").unwrap();
                    main_window.set_focus().expect("TODO: panic message");
                }
            }
            "enabled" => {
                let mut config = get_config();
                config.enabled = !config.enabled;
                set_config(config);
                if get_config().enabled {
                    item_handle.set_title("Enabled ✅").unwrap()
                } else {
                    item_handle.set_title("Enabled ❎").unwrap()
                }
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}

fn is_auth_error(err: ScrobblerError, app_handle: &AppHandle) -> bool {
    if format!("{}", err).contains("403") {
        Notification::new(app_handle.config().tauri.bundle.identifier.clone())
            .title("Last.FM Scrobbler")
            .body("Invalid Last.FM credentials. Please check them.")
            .show()
            .expect("couldnt show notification");
        true
    } else {
        println!("{}", err);
        false
    }
}

// Your secrets.
pub fn api_key() -> &'static str {
    ""
}
pub fn api_secret() -> &'static str {
    ""
}