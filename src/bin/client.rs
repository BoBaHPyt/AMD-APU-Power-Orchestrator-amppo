use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::env;
use std::process::Command;

use amppo::protocol::{DaemonReq, DaemonResp};
use serde_json::Error;

fn main() -> Result<(), Error> {
    // Проверяем аргументы. По умолчанию, если ничего не передали, шлем "toggle"
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("toggle");

    // Подключаемся к системному сокету демона
    match UnixStream::connect("/run/amppo.sock") {
        Ok(mut stream) => {
            // Отправляем команду
	    let request = DaemonReq{ cmd: command.to_string() };
	    let request_bytes = serde_json::to_vec(&request)?.to_owned();
	    
            if stream.write_all(&request_bytes).is_ok() {
                let mut response = String::new();
                // Ждем текстовый ответ (имя примененного профиля)
                if stream.read_to_string(&mut response).is_ok() {
                    // Выводим в stdout. ashell перехватит этот текст для обновления виджета
		    let des_resp: DaemonResp = serde_json::from_str(&response)?;

		    if let Some(cmd) = &des_resp.hook {
			let _ = Command::new("sh")
			    .arg("-c")
			    .arg(cmd)
			    .spawn()
			    .expect("Ошибка при выполнении команды")
			    .wait();
		    }
		    
                    println!("{}", des_resp.modename);
                }
            }
        }
        Err(_) => {
            eprintln!("Ошибка: Демон amppo не запущен или у вас нет прав доступа к сокету.");
            std::process::exit(1);
        }
    }
    Ok(())
}
