use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use amppo::protocol::{DaemonReq, DaemonResp};
use amppo::{
    config::Config,
    state::State,
    hardware::{
        cpu::update_cpu_settings,
        gpu::update_video_settings,
        platform::update_platform_profile,
    },
};

// Вспомогательная функция, чтобы не дублировать код применения настроек
fn apply_performance_mode(mode: &amppo::config::PerformanceMode) {
    let _ = update_video_settings(&mode.gpu_settings);
    let _ = update_cpu_settings(&mode.cpu_settings);
    let _ = update_platform_profile(&mode.platform_settings);
}

fn main() -> std::io::Result<()> {
    let socket_path = "/run/amppo.sock";
    let _ = fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    let mut perms = fs::metadata(socket_path)?.permissions();
    perms.set_mode(0o666);
    fs::set_permissions(socket_path, perms)?;

    // 1. Инициализация при старте
    let mut state = State::load().unwrap_or_else(|_| State::new());
    let config = Config::load("/etc/amppo/config.json").expect("Ошибка чтения конфига");

    // Получаем текущий профиль (из сохраненного состояния на SSD) и ПРИМЕНЯЕМ его к железу
    let start_mode = config.current(&mut state);
    println!("Стартовая инициализация профиля: {}", start_mode.modename);
    apply_performance_mode(start_mode);
    let _ = state.save(); // На случай, если state синхронизировался с дефолтом

    println!("AMPPO Демон успешно запущен и слушает сокет...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 128];
                if let Ok(bytes_read) = stream.read(&mut buffer) {
		    let request: DaemonReq = serde_json::from_slice(&buffer[..bytes_read])?;

                    // --- СЦЕНАРИЙ 1: КОМАНДА TOGGLE ---
                    if request.cmd == "toggle" {
                        let performance_mode = config.next(&mut state);
                        apply_performance_mode(performance_mode);
                        let _ = state.save();

			let response = DaemonResp{ modename: performance_mode.modename.clone(), hook: performance_mode.hook.clone() };
                        let response_bytes = serde_json::to_vec(&response)?.to_owned();
                        let _ = stream.write_all(&response_bytes);
                    } 
                    // --- СЦЕНАРИЙ 2: КОМАНДА CURRENT ---
                    else if request.cmd == "start" {
                        // Просто отдаем клиенту текущее имя из state (оно всегда актуально)
			let performance_mode = config.current(&mut state);
			let response = DaemonResp{ modename: performance_mode.modename.clone(), hook: performance_mode.hook.clone() };
                        let response_bytes = serde_json::to_vec(&response)?.to_owned();
                        let _ = stream.write_all(&response_bytes);
                    }
		    
		    else if request.cmd == "current" {
                        // Просто отдаем клиенту текущее имя из state (оно всегда актуально)
			let response = DaemonResp{ modename: state.current_mode.clone(), hook: None };
                        let response_bytes = serde_json::to_vec(&response)?.to_owned();
                        let _ = stream.write_all(&response_bytes);
                    }
		    state.save()?;
                }
            }
            Err(e) => eprintln!("Ошибка сокета: {}", e),
        }
    }
    Ok(())
}
