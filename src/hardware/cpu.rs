use std::{
    fmt,
    fs::OpenOptions,
    io::{Read, Write},
    ops::RangeInclusive,
    result::Result,
};
use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct CpuSettings {
    pub turbo_boost: bool,

    pub energy_performance_preference: String,
    pub scaling_governor: String,

    pub min_freq: u32,
    pub max_freq: u32,

    pub dynamic_epp: bool,
}

#[derive(Debug)]
pub enum UpdateCpuSettingsError {
    Io(std::io::Error),
    InvalidThreadsCount,
    InvalidCpuPreference,
}

// Реализуем вывод текста ошибки
impl fmt::Display for UpdateCpuSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateCpuSettingsError::Io(err) => write!(f, "IO error: {}", err),
            UpdateCpuSettingsError::InvalidThreadsCount => {
                write!(f, "Invalid threads count format")
            }
            UpdateCpuSettingsError::InvalidCpuPreference => {
                write!(
                    f,
                    "Invalid energy performance preference or scaling governor"
                )
            }
        }
    }
}

// Делаем тип официальной ошибкой Rust
impl std::error::Error for UpdateCpuSettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UpdateCpuSettingsError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// Этот трейт позволяет оператору `?` автоматически делать конвертацию
impl From<std::io::Error> for UpdateCpuSettingsError {
    fn from(err: std::io::Error) -> Self {
        UpdateCpuSettingsError::Io(err)
    }
}

fn push_cpu(path: &str, data: &str) -> Result<(), UpdateCpuSettingsError> {
    let mut file = OpenOptions::new().write(true).open(path)?;
    file.write_all(format!("{}\n", data).as_bytes())?;
    Ok(())
}

fn check_cpu(path: &str, data: &str) -> Result<(), UpdateCpuSettingsError> {
    let mut file = OpenOptions::new().read(true).open(path)?;

    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let content_str = String::from_utf8(content).unwrap_or_default();
    if content_str
        .replace("\n", "")
        .split_whitespace()
        .any(|word| word == data)
    {
        return Ok(());
    }

    Err(UpdateCpuSettingsError::InvalidCpuPreference)
}

fn get_cpu_range() -> Result<RangeInclusive<usize>, UpdateCpuSettingsError> {
    let mut file = OpenOptions::new()
        .read(true)
        .open("/sys/devices/system/cpu/present")?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let content_str = String::from_utf8(content)
        .unwrap_or(String::from("0-0"))
        .replace("\n", "");
    let cernel_range: Vec<&str> = content_str.split("-").collect();

    if cernel_range.len() == 2 {
        let start = cernel_range[0].parse::<usize>().unwrap_or_default();
        let end = cernel_range[1].parse::<usize>().unwrap_or_default();
        return Ok(start..=end);
    }
    if cernel_range.len() == 1 {
        let start = cernel_range[0].parse::<usize>().unwrap_or_default();
        return Ok(start..=start);
    }

    Err(UpdateCpuSettingsError::InvalidThreadsCount)
}

pub fn update_cpu_settings(settings: &CpuSettings) -> Result<(), UpdateCpuSettingsError> {
    let turbo_boost_path = "/sys/devices/system/cpu/cpufreq/boost";
    let dynamic_epp_path = "/sys/devices/system/cpu/amd_pstate/dynamic_epp";

    let dynamic_epp = if settings.dynamic_epp {
        "enabled"
    } else {
        "disabled"
    };

    push_cpu(dynamic_epp_path, dynamic_epp)?;
    push_cpu(turbo_boost_path, &(settings.turbo_boost as u8).to_string())?;

    for core_id in get_cpu_range()? {
        let epap_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/energy_performance_available_preferences",
            core_id
        );
        let epp_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/energy_performance_preference",
            core_id
        );
        let sag_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_available_governors",
            core_id
        );
        let sg_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_governor",
            core_id
        );
        let min_freq_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_min_freq",
            core_id
        );
        let max_freq_path = format!(
            "/sys/devices/system/cpu/cpufreq/policy{}/scaling_max_freq",
            core_id
        );

        check_cpu(&epap_path, &settings.energy_performance_preference)?;
        push_cpu(&epp_path, &settings.energy_performance_preference)?;
        
        check_cpu(&sag_path, &settings.scaling_governor)?;
        push_cpu(&sg_path, &settings.scaling_governor)?;
        
        if settings.min_freq > 0 {
            push_cpu(&min_freq_path, &settings.min_freq.to_string())?;
        }
        if settings.max_freq > 0 {
            push_cpu(&max_freq_path, &settings.max_freq.to_string())?;
        }
    }
    Ok(())
}
