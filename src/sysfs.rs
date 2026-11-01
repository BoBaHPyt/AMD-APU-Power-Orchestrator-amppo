//! Модуль для работы с файлами sysfs

use std::fmt;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::ops::RangeInclusive;
use std::path::Path;

/// Специальный тип Result для операций с sysfs
pub type SysfsResult<T> = std::result::Result<T, SysfsError>;

/// Ошибки работы с sysfs
#[derive(Debug)]
pub enum SysfsError {
    Io(std::io::Error),
    ParseError(String),
    InvalidRange,
    FileNotFound(String),
}

impl fmt::Display for SysfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SysfsError::Io(err) => write!(f, "IO error: {}", err),
            SysfsError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SysfsError::InvalidRange => write!(f, "Invalid range format"),
            SysfsError::FileNotFound(path) => write!(f, "File not found: {}", path),
        }
    }
}

impl std::error::Error for SysfsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SysfsError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SysfsError {
    fn from(err: std::io::Error) -> Self {
        SysfsError::Io(err)
    }
}

/// Прочитать содержимое файла sysfs как строку
pub fn read_sysfs_file<P: AsRef<Path>>(path: P) -> SysfsResult<String> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Err(SysfsError::FileNotFound(path.display().to_string()));
    }

    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // Удаляем завершающий символ новой строки
    let contents = contents.trim_end().to_string();
    
    Ok(contents)
}

/// Записать значение в файл sysfs
pub fn write_sysfs_file<P: AsRef<Path>>(path: P, value: &str) -> SysfsResult<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Err(SysfsError::FileNotFound(path.display().to_string()));
    }

    let mut file = OpenOptions::new().write(true).open(path)?;
    file.write_all(format!("{}\n", value).as_bytes())?;
    
    Ok(())
}

/// Прочитать содержимое файла sysfs как число u32
pub fn read_sysfs_u32<P: AsRef<Path>>(path: P) -> SysfsResult<u32> {
    let contents = read_sysfs_file(path)?;
    contents.parse::<u32>().map_err(|e| SysfsError::ParseError(e.to_string()))
}

/// Прочитать содержимое файла sysfs как число u64
pub fn read_sysfs_u64<P: AsRef<Path>>(path: P) -> SysfsResult<u64> {
    let contents = read_sysfs_file(path)?;
    contents.parse::<u64>().map_err(|e| SysfsError::ParseError(e.to_string()))
}

/// Прочитать диапазон частот из файла sysfs (формат: "min max")
pub fn read_sysfs_range<P: AsRef<Path>>(path: P) -> SysfsResult<RangeInclusive<u32>> {
    let contents = read_sysfs_file(path)?;
    let parts: Vec<&str> = contents.split_whitespace().collect();
    
    if parts.len() != 2 {
        return Err(SysfsError::InvalidRange);
    }
    
    let min = parts[0].parse::<u32>().map_err(|e| SysfsError::ParseError(e.to_string()))?;
    let max = parts[1].parse::<u32>().map_err(|e| SysfsError::ParseError(e.to_string()))?;
    
    Ok(min..=max)
}

/// Прочитать список доступных значений из файла sysfs (формат: "val1 val2 val3")
pub fn read_sysfs_list<P: AsRef<Path>>(path: P) -> SysfsResult<Vec<String>> {
    let contents = read_sysfs_file(path)?;
    Ok(contents.split_whitespace().map(|s| s.to_string()).collect())
}

/// Проверка существования файла sysfs
pub fn sysfs_file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Парсинг диапазона CPU ядер из /sys/devices/system/cpu/present
pub fn parse_cpu_range() -> SysfsResult<RangeInclusive<usize>> {
    let contents = read_sysfs_file("/sys/devices/system/cpu/present")?;
    let cernel_range: Vec<&str> = contents.split("-").collect();

    if cernel_range.len() == 2 {
        let start = cernel_range[0].parse::<usize>().map_err(|_| SysfsError::InvalidRange)?;
        let end = cernel_range[1].parse::<usize>().map_err(|_| SysfsError::InvalidRange)?;
        return Ok(start..=end);
    }
    if cernel_range.len() == 1 {
        let start = cernel_range[0].parse::<usize>().map_err(|_| SysfsError::InvalidRange)?;
        return Ok(start..=start);
    }

    Err(SysfsError::InvalidRange)
}

/// Получить список доступных governor'ов для CPU
pub fn get_available_governors(core_id: usize) -> SysfsResult<Vec<String>> {
    let path = format!("/sys/devices/system/cpu/cpufreq/policy{}/scaling_available_governors", core_id);
    read_sysfs_list(path)
}

/// Получить список доступных EPP preference
pub fn get_available_epp(core_id: usize) -> SysfsResult<Vec<String>> {
    let path = format!("/sys/devices/system/cpu/cpufreq/policy{}/energy_performance_available_preferences", core_id);
    read_sysfs_list(path)
}

/// Получить текущую минимальную и максимальную частоту CPU для ядра
pub fn get_cpu_freq_range(core_id: usize) -> SysfsResult<(u32, u32)> {
    let min_path = format!("/sys/devices/system/cpu/cpufreq/policy{}/cpuinfo_min_freq", core_id);
    let max_path = format!("/sys/devices/system/cpu/cpufreq/policy{}/cpuinfo_max_freq", core_id);
    
    let min_freq = read_sysfs_u32(&min_path)?;
    let max_freq = read_sysfs_u32(&max_path)?;
    
    Ok((min_freq, max_freq))
}

/// Получить текущие частоты GPU из pp_od_clk_voltage
pub fn get_gpu_freq_range(card: u32) -> SysfsResult<(u32, u32)> {
    let path = format!("/sys/class/drm/card{}/device/pp_od_clk_voltage", card);
    
    if !std::path::Path::new(&path).exists() {
        // Если файл не существует, пробуем получить из других источников
        return Ok((0, 0));
    }
    
    let content = read_sysfs_file(&path)?;
    
    // Парсим строку формата "OD_SCLK:" и "OD_MCLK:"
    // Формат: "OD_SCLK: 0 1000" где 0 - min, 1000 - max в MHz
    let mut min_freq = 0u32;
    let mut max_freq = 0u32;

    let mut start = false;
    for line in content.lines() {
	if line.starts_with("OD_RANGE:") {
	    start = true;
	}
	if start && line.starts_with("SCLK:") {
            let clear_line = line.replace("Mhz", "");
	    let parts: Vec<&str> = clear_line.split_whitespace().collect();
	    
            if parts.len() >= 3 {
                if let Ok(min) = parts[1].parse::<u32>() {
                    min_freq = min;
                }
                if let Ok(max) = parts[2].parse::<u32>() {
                    max_freq = max;
                }
            }
        }
    }
    // Если не удалось распарсить, возвращаем дефолтные значения
    if min_freq == 0 && max_freq == 0 {
        Ok((0, 0))
    } else {
        Ok((min_freq, max_freq))
    }
}

/// Получить текущие частоты VRAM из pp_od_clk_voltage
pub fn get_vram_freq_range(card: u32) -> SysfsResult<(u32, u32)> {
    let path = format!("/sys/class/drm/card{}/device/pp_od_clk_voltage", card);
    
    if !std::path::Path::new(&path).exists() {
        // Если файл не существует, возвращаем дефолтные значения
        return Ok((0, 3000));
    }
    
    let content = read_sysfs_file(&path)?;
    
    // Парсим строку формата "OD_MCLK:" для VRAM
    // Формат: "OD_MCLK: 0 1000" где 0 - min, 1000 - max в MHz
    let mut min_freq = 0u32;
    let mut max_freq = 3000u32;
    
    for line in content.lines() {
        if line.starts_with("OD_MCLK:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(min) = parts[1].parse::<u32>() {
                    min_freq = min;
                }
                if let Ok(max) = parts[2].parse::<u32>() {
                    max_freq = max;
                }
            }
        }
    }
    
    // Если не удалось распарсить, возвращаем дефолтные значения
    if min_freq == 0 && max_freq == 0 {
        Ok((0, 3000))
    } else {
        Ok((min_freq, max_freq))
    }
}

/// Получить доступные профили платформы
pub fn get_platform_profiles() -> SysfsResult<Vec<String>> {
    read_sysfs_list("/sys/firmware/acpi/platform_profile_choices")
}

/// Прочитать текущее значение из файла sysfs и проверить его наличие в списке доступных
pub fn read_and_validate_sysfs<P: AsRef<Path>>(path: P, available_values: &[String]) -> SysfsResult<String> {
    let value = read_sysfs_file(path)?;
    
    if available_values.contains(&value) {
        Ok(value)
    } else {
        Err(SysfsError::ParseError(format!(
            "Value '{}' not found in available values: {:?}",
            value,
            available_values
        )))
    }
}
