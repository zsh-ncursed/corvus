use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::app_state::{AppState, TabState};

/// Структура для хранения данных сессии
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionState {
    /// Список вкладок с их путями
    pub tabs: Vec<TabPath>,
    /// Индекс активной вкладки
    pub active_tab_index: usize,
    /// Показывать ли вкладки
    pub show_tabs: bool,
    /// Закладки
    pub bookmarks: Vec<(String, PathBuf)>,
    /// Показывать ли скрытые файлы
    pub show_hidden_files: bool,
}

/// Структура для хранения пути вкладки
#[derive(Debug, Serialize, Deserialize)]
pub struct TabPath {
    /// Идентификатор вкладки
    pub id: usize,
    /// Текущий каталог вкладки
    pub current_dir: PathBuf,
}

impl SessionState {
    /// Создает SessionState из AppState
    pub fn from_app_state(app_state: &AppState) -> Self {
        let tabs = app_state
            .tabs
            .iter()
            .map(|tab| TabPath {
                id: tab.id,
                current_dir: tab.current_dir.clone(),
            })
            .collect();

        Self {
            tabs,
            active_tab_index: app_state.active_tab_index,
            show_tabs: app_state.show_tabs,
            bookmarks: app_state.bookmarks.clone(),
            show_hidden_files: app_state.show_hidden_files,
        }
    }

    /// Применяет SessionState к AppState
    pub fn apply_to_app_state(&self, app_state: &mut AppState) {
        // Обновляем вкладки
        app_state.tabs.clear();
        for (_index, tab_path) in self.tabs.iter().enumerate() {
            let mut tab = TabState::new(tab_path.id);
            tab.set_current_dir(tab_path.current_dir.clone(), self.show_hidden_files);
            app_state.tabs.push(tab);
        }

        // Устанавливаем индекс активной вкладки
        app_state.active_tab_index = self.active_tab_index.min(app_state.tabs.len().saturating_sub(1));

        // Обновляем отображение вкладок
        app_state.show_tabs = self.show_tabs;

        // Обновляем закладки
        app_state.bookmarks = self.bookmarks.clone();

        // Обновляем показ скрытых файлов
        app_state.show_hidden_files = self.show_hidden_files;
        // Обновляем записи во всех вкладках в соответствии с настройкой показа скрытых файлов
        for tab in &mut app_state.tabs {
            tab.update_entries(app_state.show_hidden_files);
        }
    }
}

/// Получает путь к файлу сессии
pub fn get_session_file_path() -> PathBuf {
    let config_dir = directories::ProjectDirs::from("org", "rust-tui-fm", "rtfm")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    config_dir.join("session.json")
}

/// Сохраняет сессию в файл
pub fn save_session(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let session_state = SessionState::from_app_state(app_state);
    let session_file_path = get_session_file_path();

    // Создаем директорию конфигурации, если она не существует
    if let Some(parent) = session_file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&session_state)?;
    std::fs::write(session_file_path, json)?;
    Ok(())
}

/// Загружает сессию из файла
pub fn load_session() -> Result<Option<SessionState>, Box<dyn std::error::Error>> {
    let session_file_path = get_session_file_path();

    if !session_file_path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(session_file_path)?;
    let session_state: SessionState = serde_json::from_str(&json)?;
    Ok(Some(session_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use std::env;

    #[test]
    fn test_save_and_load_session() {
        // Создаем временную директорию для теста
        let temp_dir = TempDir::new("session_test").unwrap();
        let old_config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_default();
        
        // Устанавливаем временную директорию как директорию конфигурации
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        // Создаем тестовое состояние приложения
        let mut app_state = AppState::new();
        
        // Добавляем несколько вкладок
        app_state.new_tab();
        app_state.new_tab();
        
        // Сохраняем сессию
        assert!(save_session(&app_state).is_ok());
        
        // Проверяем, что файл сессии создан
        let session_file_path = get_session_file_path();
        assert!(session_file_path.exists());
        
        // Загружаем сессию
        let loaded_session = load_session().unwrap();
        assert!(loaded_session.is_some());
        
        let session_state = loaded_session.unwrap();
        
        // Проверяем, что данные сессии корректны
        assert_eq!(session_state.tabs.len(), 3); // 3 вкладки
        assert_eq!(session_state.active_tab_index, 2); // Активная вкладка - последняя созданная
        assert_eq!(session_state.show_tabs, true); // Вкладки должны отображаться
        
        // Восстанавливаем оригинальную директорию конфигурации
        if old_config_dir.is_empty() {
            env::remove_var("XDG_CONFIG_HOME");
        } else {
            env::set_var("XDG_CONFIG_HOME", old_config_dir);
        }
    }
}