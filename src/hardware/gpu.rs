use std::{
    fs::OpenOptions,
    io::{Result, Write},
};
use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct GpuSettings {
    pub card: u32,

    pub gpu_freq_min: u32,
    pub gpu_freq_max: u32,

    pub vram_freq_min: u32,
    pub vram_freq_max: u32,

    pub is_set: bool,
}

fn push_gpu(path: &str, data: &str, autocommit: bool) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(path)?;
    file.write_all(format!("{}\n", data).as_bytes())?;
    file.flush()?;

    if autocommit {
        file.write_all(b"c\n")?;
    }
    Ok(())
}

pub fn update_video_settings(settings: &GpuSettings) -> Result<()> {
    let performance_level_path = format!(
        "/sys/class/drm/card{}/device/power_dpm_force_performance_level",
        &settings.card
    );
    let gpu_freq_path = format!(
        "/sys/class/drm/card{}/device/pp_od_clk_voltage",
        &settings.card
    );
    //let vram_freq_path = format!("/sys/class/drm/card{}/device/pp_dpm_mclk", &settings.card); TODO:
    let vram_freq_path = format!(
        "/sys/class/drm/card{}/device/pp_od_clk_voltage",
        &settings.card
    );

    if !settings.is_set {
	push_gpu(&performance_level_path, "auto", false)?;
        return Ok(());
    }

    {
        push_gpu(&performance_level_path, "manual", false)?;
    }

    if settings.gpu_freq_min > 0 {
        push_gpu(
            &gpu_freq_path,
            &format!("s 0 {}", settings.gpu_freq_min),
            true,
        )?;
    }
    if settings.gpu_freq_max > 0 {
        push_gpu(
            &gpu_freq_path,
            &format!("s 1 {}", settings.gpu_freq_max),
            true,
        )?;
    }

    if settings.vram_freq_min > 0 {
        push_gpu(
            &vram_freq_path,
            &format!("m 0 {}", settings.vram_freq_min),
            true,
        )?;
    }
    if settings.vram_freq_max > 0 {
        push_gpu(
            &vram_freq_path,
            &format!("m 1 {}", settings.vram_freq_max),
            true,
        )?;
    }

    Ok(())
}
