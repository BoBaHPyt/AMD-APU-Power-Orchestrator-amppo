use std::{
    fmt,
    fs::OpenOptions,
    io::{Read, Write},
    result::Result,
};
use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct PlatformSettings {
    pub profile: String,
}

#[derive(Debug)]
pub enum UpdatePlatformSettingsError {
    Io(std::io::Error),
    InvalidPlatformProfile,
}

// Реализуем вывод текста ошибки
impl fmt::Display for UpdatePlatformSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdatePlatformSettingsError::Io(err) => write!(f, "IO error: {}", err),
            UpdatePlatformSettingsError::InvalidPlatformProfile => {
                write!(f, "Invalid platform profile")
            }
        }
    }
}

// Делаем тип официальной ошибкой Rust
impl std::error::Error for UpdatePlatformSettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UpdatePlatformSettingsError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// Этот трейт позволяет оператору `?` автоматически делать конвертацию
impl From<std::io::Error> for UpdatePlatformSettingsError {
    fn from(err: std::io::Error) -> Self {
        UpdatePlatformSettingsError::Io(err)
    }
}

fn push_platform(path: &str, data: &str) -> Result<(), UpdatePlatformSettingsError> {
    let mut file = OpenOptions::new().write(true).open(path)?;
    file.write_all(format!("{}\n", data).as_bytes())?;
    Ok(())
}

fn check_platform(path: &str, data: &str) -> Result<(), UpdatePlatformSettingsError> {
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
    
    Err(UpdatePlatformSettingsError::InvalidPlatformProfile)
}

pub fn update_platform_profile(settings: &PlatformSettings) -> Result<(), UpdatePlatformSettingsError> {
    let platform_profile_choices = "/sys/firmware/acpi/platform_profile_choices";
    let platform_profile = "/sys/firmware/acpi/platform_profile";

    check_platform(platform_profile_choices, &settings.profile)?;
    push_platform(platform_profile, &settings.profile)?;
    Ok(())
}
