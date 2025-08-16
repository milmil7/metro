// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_installed_shells() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        use std::fs;

        let path_env = env::var("PATH").unwrap_or_default();
        let paths: Vec<&str> = path_env.split(';').collect();

        let known_shells = vec![
            "powershell.exe",
            "pwsh.exe",
            "cmd.exe",
            "wsl.exe",
            "bash.exe",
            "zsh.exe",
            "fish.exe",
            "nu.exe",
            "git.exe",
        ];

        let mut found_shells = vec![];

        for shell in known_shells {
            for dir in &paths {
                let candidate = format!("{}/{}", dir.trim_end_matches('\\'), shell);
                if fs::metadata(&candidate).is_ok() {
                    found_shells.push(shell.replace(".exe","").to_string());
                    break;
                }
            }
        }

        found_shells
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::fs;

        let contents = fs::read_to_string("/etc/shells").unwrap_or_default();

        contents
            .lines()
            .filter(|line| line.starts_with('/'))
            .map(|line| line.to_string())
            .collect()
    }
}
use tauri::Emitter;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use tauri::Listener;
struct LaunchArg(Mutex<Option<String>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(512)
        .worker_threads(128)
        .build()
        .unwrap();
    tauri::async_runtime::set(runtime.handle().clone());

    tauri::Builder::default()
        .manage(LaunchArg(Mutex::new(std::env::args().nth(1))))
        .setup(|app| {
            let handle = app.handle(); // AppHandle (owned, cloneable)
            let state = app.state::<LaunchArg>();
            let arg = state.0.lock().unwrap().take();

            let handle_clone = handle.clone(); // ✅ clone it for use in closure

            handle.listen("tauri://ready", move |_| {
                if let Some(path) = arg.clone() {
                    let _ = handle_clone.emit("open-folder", path);
                }else {
                    let _ = handle_clone.emit("open-folder", "none");
                }
            });

            Ok(())
        })

        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_pty::init())
        .invoke_handler(tauri::generate_handler![greet, get_installed_shells])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
