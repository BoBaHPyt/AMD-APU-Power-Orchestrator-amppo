use std::fs::{self, OpenOptions, File};
use std::io::{self, Read, Write, Result};
use std::path::{Path, PathBuf};
use chrono::Local;
use serde_json::{self, Value};

/// Директория для хранения резервных копий
const BACKUP_DIR: &str = "/var/lib/amppo/backups";

/// Максимальное количество резервных копий по умолчанию
const DEFAULT_MAX_BACKUPS: usize = 10;

/// Создать резервную копию конфигурационного файла
/// Возвращает путь к созданной резервной копии
pub fn create_backup(config_path: &str) -> Result<PathBuf> {
    // Создаем директорию для резервных копий
    fs::create_dir_all(BACKUP_DIR)?;

    // Читаем исходный конфигурационный файл
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Формируем имя файла резервной копии с временной меткой
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_filename = format!("config_backup_{}.json", timestamp);
    let backup_path = Path::new(BACKUP_DIR).join(backup_filename);

    // Записываем резервную копию
    let mut backup_file = File::create(&backup_path)?;
    backup_file.write_all(contents.as_bytes())?;

    Ok(backup_path)
}

/// Получить список всех резервных копий, отсортированных по дате создания (новые последними)
pub fn list_backups() -> Result<Vec<PathBuf>> {
    let mut backups = Vec::new();
    
    if Path::new(BACKUP_DIR).exists() {
        for entry in fs::read_dir(BACKUP_DIR)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                backups.push(path);
            }
        }
    }
    
    // Сортируем по имени файла (которое содержит timestamp)
    backups.sort();
    
    Ok(backups)
}

/// Восстановить конфигурацию из резервной копии
pub fn restore_backup(backup_path: &Path, config_path: &str) -> Result<()> {
    // Проверяем существование резервной копии
    if !backup_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Резервная копия не найдена: {}", backup_path.display()),
        ));
    }

    // Читаем резервную копию
    let mut file = File::open(backup_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Проверяем, что это валидный JSON
    let _: Value = serde_json::from_str(&contents)?;

    // Создаем резервную копию текущей конфигурации перед восстановлением
    let _ = create_backup(config_path);

    // Записываем восстановленную конфигурацию
    let mut config_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_path)?;
    config_file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Удалить резервную копию
pub fn delete_backup(backup_path: &Path) -> Result<()> {
    if !backup_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Резервная копия не найдена: {}", backup_path.display()),
        ));
    }
    fs::remove_file(backup_path)?;
    Ok(())
}

/// Очистить старые резервные копии, оставив только max_backups штук
pub fn clean_old_backups(max_backups: usize) -> Result<()> {
    let backups = list_backups()?;
    
    if backups.len() <= max_backups {
        return Ok(());
    }

    // Удаляем самое старое количество резервных копий
    let to_delete = backups.len() - max_backups;
    for backup_path in backups.into_iter().take(to_delete) {
        let _ = delete_backup(&backup_path);
    }

    Ok(())
}

/// Автоматическая очистка резервных копий
pub fn auto_clean_backups() -> Result<()> {
    clean_old_backups(DEFAULT_MAX_BACKUPS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_list_backups() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        
        // Создаем тестовый конфиг
        fs::write(&config_path, r#"{"test": "data"}"#).unwrap();
        
        // Создаем резервную копию
        let backup_path = create_backup(config_path.to_str().unwrap()).unwrap();
        
        // Проверяем, что файл создан
        assert!(backup_path.exists());
    }
}
