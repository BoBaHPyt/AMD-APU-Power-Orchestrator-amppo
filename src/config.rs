use std::{fs::OpenOptions, io::Result};

use crate::{hardware::{cpu::CpuSettings, gpu::GpuSettings, platform::PlatformSettings}, state::State};
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

    pub fn current<'a>(&'a self, state: &mut State) -> &'a PerformanceMode {
	if self.modes.len() <= state.index_mod {
	    state.index_mod = 0;
	}

	self.modes.get(state.index_mod).unwrap()
    }

    pub fn next<'a>(&'a self, state: &mut State) -> &'a PerformanceMode {
	state.index_mod += 1;
	if self.modes.len() <= state.index_mod {
	    state.index_mod = 0;
	}

	let mode = self.modes.get(state.index_mod).unwrap();
	state.current_mode = mode.modename.clone();

	mode
    }
}
