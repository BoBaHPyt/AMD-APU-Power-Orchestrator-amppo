use std::process::Command;

use crate::{config::Config, hardware::{
    cpu::update_cpu_settings,
    gpu::update_video_settings,
    platform::update_platform_profile,
}};

mod hardware;
mod config;

fn main() {
    let config = Config::load("./config.json").expect("Ошибка чтения конфига");

    let performance_mode = config.next();

    println!("Применение {}", performance_mode.modename);

    if let Some(hook) = &performance_mode.hook {
	Command::new("sh")
            .arg("-c")
            .arg(hook).spawn().expect("Ошибка при выполнении команды").wait().expect("Ошибка при выполнении команды");
    }
    
    update_video_settings(&performance_mode.gpu_settings).expect("Ошибка обновления параметров gpu");
    update_cpu_settings(&performance_mode.cpu_settings).expect("Ошибка обновления параметров cpu");
    update_platform_profile(&performance_mode.platform_settings).expect("Ошибка обновления параметров platform");
}
