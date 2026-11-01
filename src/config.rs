use std::{fs::{OpenOptions, File}, io::Result};

use crate::{hardware::{cpu::CpuSettings, gpu::GpuSettings, platform::PlatformSettings}, state::State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PerformanceMode {
    pub modename: String,
    
    pub cpu_settings: CpuSettings,
    pub gpu_settings: GpuSettings,
    pub platform_settings: PlatformSettings,
    pub hook: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub modes: Vec<PerformanceMode>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
	let file = OpenOptions::new().read(true).open(path)?;

	Ok(serde_json::from_reader(file)?)
    }

    pub fn reload(&mut self, path: &str) -> Result<()> {
	let file = OpenOptions::new().read(true).open(path)?;

	*self = serde_json::from_reader(file)?;

	Ok(())
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

    pub fn find_mode(&self, modename: &str) -> Option<&PerformanceMode> {
	self.modes.iter().find(|mode| mode.modename == modename)
    }

    pub fn find_mode_index(&self, modename: &str) -> Option<usize> {
	self.modes.iter().position(|mode| mode.modename == modename)
    }

    pub fn save(&self, path: &str) -> Result<()> {
	let file = File::create(path)?;
	serde_json::to_writer_pretty(file, self)?;
	Ok(())
    }
}
