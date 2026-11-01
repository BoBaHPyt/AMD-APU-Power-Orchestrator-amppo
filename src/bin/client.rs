use std::os::unix::net::UnixStream;
use std::io::{self, Read, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::path::PathBuf;

use amppo::protocol::{CliResp, DaemonReq, DaemonResp};
use amppo::backup::{create_backup, list_backups, restore_backup, delete_backup, clean_old_backups};
use amppo::tui::run_tui_editor;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(author, version, about = "CLI для управления amppo-daemon")]
struct Cli {
    #[arg(long, global = true, help = "Выводить результат в формате JSON")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Показать текущий профиль питания")]
    Current {
        #[arg(
            long, 
            value_name = "SECONDS", 
            default_missing_value = "1", 
            num_args = 0..=1,
            help = "Циклический вывод каждые N секунд"
        )]
        r#while: Option<u64>,
    },

    #[command(about = "Переключить профиль на следующий")]
    Toggle,

    #[command(about = "Установить текущий профиль питания (например после перезагрузки пк)")]
    Start,

    #[command(about = "Перезагрузить конфиг")]
    Reload,

    #[command(about = "Установить указанный профиль питания по имени")]
    Set {
        #[arg(value_name = "PROFILE_NAME", help = "Имя профиля из конфигурации")]
        modename: String,
    },

    #[command(about = "Создать резервную копию текущей конфигурации")]
    Dump,

    #[command(about = "Редактировать профили (TUI интерфейс)")]
    Edit,

    #[command(about = "Управление резервными копиями")]
    Backups {
        #[command(subcommand)]
        command: BackupCommands,
    }
}

#[derive(Subcommand)]
enum BackupCommands {
    #[command(about = "Показать список резервных копий")]
    List,

    #[command(about = "Восстановить конфигурацию из резервной копии")]
    Restore {
        #[arg(value_name = "BACKUP_FILE", help = "Имя файла резервной копии")]
        backup_file: String,
    },

    #[command(about = "Удалить резервную копию")]
    Delete {
        #[arg(value_name = "BACKUP_FILE", help = "Имя файла резервной копии")]
        backup_file: String,
    },

    #[command(about = "Очистить старые резервные копии")]
    Clean,
}


fn send_request(request: &DaemonReq) -> Result<DaemonResp, io::Error> {
    match UnixStream::connect("/run/amppo.sock") {
        Ok(mut stream) => {
            let request_bytes = serde_json::to_vec(&request)?.to_owned();
            
            stream.write_all(&request_bytes)?;
            let mut response = String::new();
            stream.read_to_string(&mut response)?;
            let des_resp: DaemonResp = serde_json::from_str(&response)?;

            Ok(des_resp)
        }
        Err(_) => {
            eprintln!("Ошибка: Демон amppo не запущен или у вас нет прав доступа к сокету.");
            std::process::exit(1);
        }
    }
}

fn process_resp(resp: &DaemonResp) {
    if let Some(cmd) = &resp.hook {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .expect("Ошибка при выполнении команды")
            .wait();
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            let request = DaemonReq{ cmd: String::from("start") };
            let resp = send_request(&request).expect("Ошибка во время чтения/записи сокета");
            process_resp(&resp);
            
            if cli.json {
                let cli_result = CliResp { text: resp.modename, alt: String::new() };
                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
            } else {
                println!("{}", resp.modename);
            }
        }
        Commands::Toggle => {
            let request = DaemonReq{ cmd: String::from("toggle") };
            let resp = send_request(&request).expect("Ошибка во время чтения/записи сокета");
            process_resp(&resp);
            
            if cli.json {
                let cli_result = CliResp { text: resp.modename, alt: String::new() };
                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
            } else {
                println!("{}", resp.modename);
            }
        }
        Commands::Current { r#while } => {
            let request = DaemonReq{ cmd: String::from("current") };

            if let Some(seconds) = r#while {
                loop {
                    let resp = send_request(&request).expect("Ошибка во время чтения/записи сокета");
                    
                    if cli.json {
                        let cli_result = CliResp { text: resp.modename, alt: String::new() };
                        println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                    } else {
                        println!("{}", resp.modename);
                    }
                    sleep(Duration::from_secs(*seconds));
                }
            } else {
                let resp = send_request(&request).expect("Ошибка во время чтения/записи сокета");
                
                if cli.json {
                    let cli_result = CliResp { text: resp.modename, alt: String::new() };
                    println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                } else {
                    println!("{}", resp.modename);
                }
            }
        }
	Commands::Reload => {
	    let request = DaemonReq { cmd: String::from("reload") };
            let resp = send_request(&request).expect("Ошибка во время чтения/записи сокета");
            
            if cli.json {
                let cli_result = CliResp { text: resp.modename, alt: String::new() };
                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
            } else {
                println!("{}", resp.modename);
            }
	}
        Commands::Set { modename } => {
            let request = DaemonReq { cmd: format!("set {}", modename) };
            let resp = send_request(&request).expect("Ошибка во время чтения/записи сокета");
            process_resp(&resp);
            
            if cli.json {
                let cli_result = CliResp { text: resp.modename, alt: String::new() };
                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
            } else {
                println!("{}", resp.modename);
            }
        }
        Commands::Dump => {
            let config_path = "/etc/amppo/config.json";
            match create_backup(config_path) {
                Ok(backup_path) => {
                    if cli.json {
                        let cli_result = CliResp { 
                            text: format!("Резервная копия создана: {}", backup_path.display()),
                            alt: String::new() 
                        };
                        println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                    } else {
                        println!("Резервная копия создана: {}", backup_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка при создании резервной копии: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Edit => {
            let config_path = "/etc/amppo/config.json";
            // Проверяем, что файл существует и у нас есть права
            if PathBuf::from(config_path).exists() {
                if let Err(e) = run_tui_editor(config_path) {
                    eprintln!("Ошибка при запуске редактора: {}", e);
                    std::process::exit(1);
                }
		let request = DaemonReq { cmd: String::from("reload") };
		send_request(&request).expect("Ошибка во время чтения/записи сокета");
            } else {
                eprintln!("Конфигурационный файл не найден: {}", config_path);
                eprintln!("Создайте конфигурацию с помощью: sudo cp example_config.json /etc/amppo/config.json");
                std::process::exit(1);
            }
        }
        Commands::Backups { command } => {
            match command {
                BackupCommands::List => {
                    match list_backups() {
                        Ok(backups) => {
                            if backups.is_empty() {
                                if cli.json {
                                    let cli_result = CliResp { text: String::from("Нет резервных копий"), alt: String::new() };
                                    println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                                } else {
                                    println!("Нет резервных копий");
                                }
                            } else {
                                if cli.json {
                                    let backup_names: Vec<String> = backups.iter()
                                        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
                                        .collect();
                                    let cli_result = CliResp { 
                                        text: format!("Найдено {} резервных копий", backups.len()),
                                        alt: serde_json::to_string(&backup_names).unwrap_or_default()
                                    };
                                    println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                                } else {
                                    println!("Доступные резервные копии:");
                                    for backup in backups {
                                        println!("  - {}", backup.file_name().unwrap_or_default().to_string_lossy());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка при получении списка резервных копий: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                BackupCommands::Restore { backup_file } => {
                    let backup_path = PathBuf::from("/var/lib/amppo/backups/").join(backup_file);
                    let config_path = "/etc/amppo/config.json";
                    match restore_backup(&backup_path, config_path) {
                        Ok(_) => {
                            if cli.json {
                                let cli_result = CliResp { 
                                    text: format!("Конфигурация восстановлена из {}", backup_file),
                                    alt: String::new() 
                                };
                                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                            } else {
                                println!("Конфигурация восстановлена из {}", backup_file);
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка при восстановлении: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                BackupCommands::Delete { backup_file } => {
                    let backup_path = PathBuf::from("/var/lib/amppo/backups/").join(backup_file);
                    match delete_backup(&backup_path) {
                        Ok(_) => {
                            if cli.json {
                                let cli_result = CliResp { 
                                    text: format!("Резервная копия {} удалена", backup_file),
                                    alt: String::new() 
                                };
                                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                            } else {
                                println!("Резервная копия {} удалена", backup_file);
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка при удалении: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                BackupCommands::Clean => {
                    match clean_old_backups(10) {
                        Ok(_) => {
                            if cli.json {
                                let cli_result = CliResp { 
                                    text: String::from("Старые резервные копии очищены"),
                                    alt: String::new() 
                                };
                                println!("{}", serde_json::to_string(&cli_result).unwrap_or_else(|_| String::from("{}")));
                            } else {
                                println!("Старые резервные копии очищены");
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка при очистке: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
}
