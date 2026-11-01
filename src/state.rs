use std::{fs::OpenOptions, io::Result};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub index_mod: usize,
    pub current_mode: String
}

impl Default for State {
    fn default() -> Self {
	Self::new()
    }
}

impl State {
    pub fn new() -> Self {
	Self { index_mod: 0, current_mode: String::new() }
    }
    
    pub fn load() -> Result<Self> {
	let file = OpenOptions::new().read(true).open("/var/lib/amppo/state.json")?;
	
	Ok(serde_json::from_reader(file)?)
    }

    pub fn save(&self) -> Result<()> {
	let file = OpenOptions::new().write(true).create(true).truncate(true).open("/var/lib/amppo/state.json")?;
	
	serde_json::to_writer(file, self)?;
	Ok(())
    }
}
