use std::{fs::OpenOptions, io::Result};

use crate::hardware::{cpu::CpuSettings, gpu::GpuSettings, platform::PlatformSettings};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PerformanceMode {
    pub modename: String,
    
    pub cpu_settings: CpuSettings,
    pub gpu_settings: GpuSettings,
    pub platform_settings: PlatformSettings,
    pub hook: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub modes: Vec<PerformanceMode>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
	let file = OpenOptions::new().read(true).open(path)?;

	Ok(serde_json::from_reader(file)?)
    }

    pub fn next(&self) -> &PerformanceMode {
	&self.modes[0]
    }
}
