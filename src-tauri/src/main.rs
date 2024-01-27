// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tauri::Manager;

const RTID: leaf::RuntimeId = 0;

static IS_LISTENING_COMMANDS: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref REMOTE_SOCKS_IP: Mutex<Option<String>> = Mutex::new(None);
    static ref REMOTE_SOCKS_PORT: Mutex<Option<u16>> = Mutex::new(None);
    static ref LISTEN_ADDRESS: Mutex<Option<String>> = Mutex::new(None);
    static ref LISTEN_IP: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Deserialize, Clone, Serialize)]
struct AcceptSocks {
    ip: String,
    port: u16,
}

async fn accept_socks(
    State(app_handle): State<tauri::AppHandle>,
    Json(payload): Json<AcceptSocks>,
) -> impl IntoResponse {
    *REMOTE_SOCKS_IP.lock().unwrap() = Some(payload.ip);
    *REMOTE_SOCKS_PORT.lock().unwrap() = Some(payload.port);
    app_handle.emit_all("start_vpn", ()).unwrap();
    (StatusCode::OK, ())
}

async fn get_proxy_pac() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/x-ns-proxy-autoconfig")
        .body(String::from(
            "function FindProxyForURL(url, host) { return \"SOCKS5 127.0.0.1:1080\" }",
        ))
        .unwrap()
}

async fn serve_http(app_handle: tauri::AppHandle, listen_addr: String) {
    let app = Router::new()
        .route("/accept_socks", post(accept_socks))
        .route("/proxy.pac", get(get_proxy_pac))
        .with_state(app_handle);
    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tauri::command]
fn listen_commands(app_handle: tauri::AppHandle) {
    let iface = default_net::get_default_interface().unwrap();
    let ip = iface.ipv4.first().unwrap().addr;
    let listen_addr = format!("{}:1180", &ip);
    let url = format!(
        "leafvpn://connect_socks?url=http://{}/accept_socks",
        &listen_addr
    );
    *LISTEN_ADDRESS.lock().unwrap() = Some(url);
    *LISTEN_IP.lock().unwrap() = Some(ip.to_string());
    if !IS_LISTENING_COMMANDS.load(Ordering::Relaxed) {
        thread::spawn(move || {
            IS_LISTENING_COMMANDS.store(true, Ordering::Relaxed);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(serve_http(app_handle, listen_addr));
        });
    }
}

#[tauri::command]
fn start_vpn(app_handle: tauri::AppHandle, socks_ip: String, socks_port: u16) {
    let config = format!("[General]\nloglevel=debug\ndns-server=1.1.1.1\nsocks-interface=127.0.0.1\nsocks-port=1080\n[Proxy]\nproxy=socks,{},{}", socks_ip, socks_port);
    thread::spawn(move || {
        if !leaf::is_running(RTID) {
            enable_system_proxy(app_handle);
            println!("starting leaf with config: {}", &config);
            let opts = leaf::StartOptions {
                config: leaf::Config::Str(config),
                runtime_opt: leaf::RuntimeOption::SingleThread,
            };
            if let Err(e) = leaf::start(RTID, opts) {
                println!("start leaf failed: {}", e);
            }
        }
    });
}

#[tauri::command]
fn is_vpn_running() -> bool {
    leaf::is_running(RTID)
}

#[tauri::command]
fn get_remote_socks_ip() -> String {
    REMOTE_SOCKS_IP.lock().unwrap().clone().unwrap()
}

#[tauri::command]
fn get_remote_socks_port() -> u16 {
    REMOTE_SOCKS_PORT.lock().unwrap().unwrap()
}

#[tauri::command]
fn get_listen_address() -> String {
    LISTEN_ADDRESS.lock().unwrap().clone().unwrap()
}

fn get_listen_ip() -> String {
    LISTEN_IP.lock().unwrap().clone().unwrap()
}

#[tauri::command]
fn stop_vpn(app_handle: tauri::AppHandle) {
    leaf::shutdown(RTID);
    disable_system_proxy(app_handle);
}

#[tauri::command]
fn is_debug() -> bool {
    #[cfg(debug_assertions)]
    {
        true
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

fn configure_proxy_file(app_handle: tauri::AppHandle) -> Option<String> {
    let f = if cfg!(target_os = "macos") {
        String::from("scripts/configure_proxy")
    } else if cfg!(target_os = "windows") {
        String::from("scripts/configure_proxy.bat")
    } else {
        panic!("unsupported os");
    };
    let pb = app_handle.path_resolver().resolve_resource(f)?;
    let pb = dunce::canonicalize(pb).ok()?;
    let p = pb.to_str()?;
    Some(p.to_string())
}

fn enable_system_proxy(app_handle: tauri::AppHandle) {
    let config_proxy = configure_proxy_file(app_handle).unwrap();
    if cfg!(target_os = "macos") {
        Command::new("sh")
            .args([&config_proxy, "on", "0", "1080"])
            .status()
            .expect("unable to configure proxy");
    } else if cfg!(target_os = "windows") {
        let ip = get_listen_ip();
        Command::new("cmd")
            .args(["/C", &config_proxy, "on", &ip, "1180"])
            .status()
            .expect("unable to configure proxy");
    } else {
        panic!("unsupported os");
    }
}

fn disable_system_proxy(app_handle: tauri::AppHandle) {
    let config_proxy = configure_proxy_file(app_handle).unwrap();
    if cfg!(target_os = "macos") {
        Command::new("sh")
            .args([&config_proxy, "off", "0", "1080"])
            .status()
            .expect("unable to configure proxy");
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &config_proxy, "off"])
            .status()
            .expect("unable to configure proxy");
    } else {
        panic!("unsupported os");
    }
}

fn main() {
    tauri::Builder::default()
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Destroyed => {
                disable_system_proxy(event.window().app_handle());
            }
            _ => {}
        })
        .setup(|app| {
            disable_system_proxy(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            listen_commands,
            start_vpn,
            is_vpn_running,
            get_remote_socks_ip,
            get_remote_socks_port,
            get_listen_address,
            stop_vpn,
            is_debug,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
