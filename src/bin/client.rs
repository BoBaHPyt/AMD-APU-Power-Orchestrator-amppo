use std::os::unix::net::UnixStream;
use std::io::{self, Read, Write};
use std::env;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use amppo::protocol::{CliResp, DaemonReq, DaemonResp};
use clap::{Parser, Subcommand};
use serde_json::Error;


#[derive(Parser)]
#[command(author, version, about = "CLI для управления amppo-daemon")]
struct Cli {
    // Глобальный флаг --json. Будет доступен в любой команде.
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
    Start
}


fn send_request(request: &DaemonReq) -> Result<DaemonResp, io::Error> {
    // Подключаемся к системному сокету демона
    match UnixStream::connect("/run/amppo.sock") {
        Ok(mut stream) => {
	    let request_bytes = serde_json::to_vec(&request)?.to_owned();
	    
            stream.write_all(&request_bytes)?;
            let mut response = String::new();
            // Ждем текстовый ответ (имя примененного профиля)
            stream.read_to_string(&mut response)?;
            // Выводим в stdout. ashell перехватит этот текст для обновления виджета
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
	    }else {
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
	    }else {
		println!("{}", resp.modename);
	    }
	}
	Commands::Current { r#while } => {
	    let request = DaemonReq{ cmd: String::from("current") };

	    if let Some(seconds) = r#while {
		while true {
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
    }
}
