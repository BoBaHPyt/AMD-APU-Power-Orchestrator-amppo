use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;

use amppo::protocol::{DaemonReq, DaemonResp};
use amppo::{
    config::Config,
    hardware::{
        cpu::update_cpu_settings, gpu::update_video_settings, platform::update_platform_profile,
    },
    state::State,
};

// Вспомогательная функция, чтобы не дублировать код применения настроек
fn apply_performance_mode(mode: &amppo::config::PerformanceMode) {
    let _ = update_video_settings(&mode.gpu_settings).expect("Не удалось обновить настройки gpu");
    let _ = update_cpu_settings(&mode.cpu_settings).expect("Не удалось обновить настройки cpu");
    let _ = update_platform_profile(&mode.platform_settings).expect("Не удалось обновить настройки platform");
}

fn main() -> std::io::Result<()> {
    let config_paht = "/etc/amppo/config.json";
    
    let socket_path = "/run/amppo.sock";
    let _ = fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    let mut perms = fs::metadata(socket_path)?.permissions();
    perms.set_mode(0o666);
    fs::set_permissions(socket_path, perms)?;

    // Инициализация при старте
    let mut state = State::load().unwrap_or_else(|_| State::new());
    let mut config = Config::load(config_paht).expect("Ошибка чтения конфига");

    // Получаем текущий профиль (из сохраненного состояния на SSD) и ПРИМЕНЯЕМ его к железу
    let start_mode = config.current(&mut state);
    println!("Стартовая инициализация профиля: {}", start_mode.modename);
    apply_performance_mode(start_mode);
    let _ = state.save();

    println!("AMPPO Демон успешно запущен и слушает сокет...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 128];
                if let Ok(bytes_read) = stream.read(&mut buffer) {
                    let request: DaemonReq = serde_json::from_slice(&buffer[..bytes_read])?;

                    // СЦЕНАРИЙ 1: КОМАНДА TOGGLE
                    if request.cmd == "toggle" {
                        let performance_mode = config.next(&mut state);
                        apply_performance_mode(performance_mode);
                        let _ = state.save();

                        let response = DaemonResp {
                            modename: performance_mode.modename.clone(),
                            hook: performance_mode.hook.clone(),
                        };
                        let response_bytes = serde_json::to_vec(&response)?.to_owned();
                        let _ = stream.write_all(&response_bytes);
                    }
                    // СЦЕНАРИЙ 2: КОМАНДА START
                    else if request.cmd == "start" {
                        let performance_mode = config.current(&mut state);
                        let response = DaemonResp {
                            modename: performance_mode.modename.clone(),
                            hook: performance_mode.hook.clone(),
                        };
                        let response_bytes = serde_json::to_vec(&response)?.to_owned();
                        let _ = stream.write_all(&response_bytes);
                    }
                    // СЦЕНАРИЙ 3: КОМАНДА CURRENT
                    else if request.cmd == "current" {
                        let response = DaemonResp {
                            modename: state.current_mode.clone(),
                            hook: None,
                        };
                        let response_bytes = serde_json::to_vec(&response)?.to_owned();
                        let _ = stream.write_all(&response_bytes);
                    }
                    // СЦЕНАРИЙ 4: КОМАНДА SET
                    else if request.cmd.starts_with("set ") {
                        let modename = request.cmd.trim_start_matches("set ");
                        if let Some(mode_index) = config.find_mode_index(modename) {
                            state.index_mod = mode_index;
                            state.current_mode = modename.to_string();
                            let performance_mode = config.current(&mut state);
                            apply_performance_mode(performance_mode);
                            let _ = state.save();

                            let response = DaemonResp {
                                modename: performance_mode.modename.clone(),
                                hook: performance_mode.hook.clone(),
                            };
                            let response_bytes = serde_json::to_vec(&response)?.to_owned();
                            let _ = stream.write_all(&response_bytes);
                        } else {
                            eprintln!("Ошибка: Профиль '{}' не найден в конфигурации", modename);
                            let error_response = DaemonResp {
                                modename: String::from("error"),
                                hook: None,
                            };
                            let response_bytes = serde_json::to_vec(&error_response)?.to_owned();
                            let _ = stream.write_all(&response_bytes);
                        }
                    }
		    // СЦЕНАРИЙ 5: КОМАНДА reload
                    else if request.cmd == "reload" {
                        config.reload(config_paht)?;

			let error_response = DaemonResp {
                                modename: String::from("success reload"),
                                hook: None,
                            };
                        let response_bytes = serde_json::to_vec(&error_response)?.to_owned();
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
