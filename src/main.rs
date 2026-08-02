use std::{
    fs::OpenOptions,
    io::{Read, Result, Write},
    ops::RangeInclusive,
};

struct VideoSettings {
    card: u32,

    gpu_freq_min: u32,
    gpu_freq_max: u32,

    vram_freq_min: u32,
    vram_freq_max: u32,

    is_set: bool,
}

struct CpuSettings {
    turbo_boost: bool,
    energy_performance_preference: String,
    scaling_governor: String,
    min_freq: u32,
    max_freq: u32,
    dynamic_epp: bool,
}

struct PlatformSettings {
    profile: String,
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

fn push_cpu(path: &str, data: &str) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(path)?;
    file.write_all(format!("{}\n", data).as_bytes())?;
    Ok(())
}

fn check_cpu(path: &str, data: &str) -> Result<bool> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut content: Vec<u8> = Vec::new();
    if file.read_to_end(&mut content).is_ok() {
        let content_str = String::from_utf8(content).unwrap_or_default();
        let result = content_str
            .replace("\n", "")
            .split_whitespace()
            .any(|word| word == data);
        return Ok(result);
    }
    Ok(false)
}

fn get_cpu_range() -> Option<RangeInclusive<usize>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open("/sys/devices/system/cpu/present")
        .unwrap();
    let mut content: Vec<u8> = Vec::new();
    if file.read_to_end(&mut content).is_ok() {
        let content_str = String::from_utf8(content)
            .unwrap_or(String::from("0-0"))
            .replace("\n", "");
        let cernel_range: Vec<&str> = content_str.split("-").collect();
        if cernel_range.len() == 2 {
            let start = cernel_range[0].parse::<usize>().unwrap_or_default();
            let end = cernel_range[1].parse::<usize>().unwrap_or_default();
            return Some(start..=end);
        }
        if cernel_range.len() == 1 {
            let start = cernel_range[0].parse::<usize>().unwrap_or_default();
            return Some(start..=start);
        }
    }
    None
}

fn update_video_settings(settings: &VideoSettings) -> Result<()> {
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
        let mut file = OpenOptions::new()
            .write(true)
            .open(performance_level_path)?;
        file.write_all(b"auto\n")?;
        return Ok(());
    }

    {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&performance_level_path)?;
        file.write_all(b"manual\n")?;
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

fn update_cpu_settings(settings: &CpuSettings) -> Result<()> {
    let turbo_boost_path = "/sys/devices/system/cpu/cpufreq/boost";
    let dynamic_epp_path = "/sys/devices/system/cpu/amd_pstate/dynamic_epp";

    let dynamic_epp = if settings.dynamic_epp {
        "enabled"
    } else {
        "disabled"
    };

    push_cpu(dynamic_epp_path, dynamic_epp)?;
    push_cpu(turbo_boost_path, &(settings.turbo_boost as u8).to_string())?;

    if let Some(cpu_range) = get_cpu_range() {
        for core_id in cpu_range {
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

            if check_cpu(&epap_path, &settings.energy_performance_preference).unwrap_or(false) {
                push_cpu(&epp_path, &settings.energy_performance_preference)?;
            }
            if check_cpu(&sag_path, &settings.scaling_governor).unwrap_or(false) {
                push_cpu(&sg_path, &settings.scaling_governor)?;
            }
            if settings.min_freq > 0 {
                push_cpu(&min_freq_path, &settings.min_freq.to_string())?;
            }
            if settings.max_freq > 0 {
                push_cpu(&max_freq_path, &settings.max_freq.to_string())?;
            }
        }
    } else {
        println!("Ошибка: не удалось распарсить диапазот cpu");
    }

    Ok(())
}

fn update_platform_profile(settings: &PlatformSettings) -> Result<()> {
    let platform_profile_choices = "/sys/firmware/acpi/platform_profile_choices";
    let platform_profile = "/sys/firmware/acpi/platform_profile";

    if check_cpu(platform_profile_choices, &settings.profile).is_ok() {
        push_cpu(platform_profile, &settings.profile)?;
    }
    Ok(())
}

fn main() {
    let gpu_settings = VideoSettings {
        card: 1,
        gpu_freq_min: 0,
        gpu_freq_max: 1700,
        vram_freq_min: 0,
        vram_freq_max: 0,
        is_set: true,
    };

    let cpu_settings = CpuSettings {
        turbo_boost: false,
        energy_performance_preference: String::from("power"),
        scaling_governor: String::from("powersave"),
        min_freq: 0,
        max_freq: 3_200_000,
        dynamic_epp: false,
    };

    let platform_settings = PlatformSettings {
        profile: String::from("low-power"),
    };

    update_video_settings(&gpu_settings).expect("Ошибка записи в файл видеокарты");
    update_cpu_settings(&cpu_settings).expect("Ошибка записи в файл процессора");
    update_platform_profile(&platform_settings).expect("Ошибка записи в файл платформы");
}
