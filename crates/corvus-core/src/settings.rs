use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Цветовые схемы
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ColorScheme {
    // Светлые темы
    GithubLight,
    SolarizedLight,
    AtomLight,
    VisualStudioLight,
    // Тёмные темы
    Dracula,
    Monokai,
    OneDark,
    Nord,
    GruvboxDark,
    TokyoNight,
    MaterialDark,
    Cyberpunk,
}

impl ColorScheme {
    pub fn name(&self) -> &'static str {
        match self {
            ColorScheme::GithubLight => "GitHub Light",
            ColorScheme::SolarizedLight => "Solarized Light",
            ColorScheme::AtomLight => "Atom Light",
            ColorScheme::VisualStudioLight => "Visual Studio Light",
            ColorScheme::Dracula => "Dracula",
            ColorScheme::Monokai => "Monokai",
            ColorScheme::OneDark => "One Dark",
            ColorScheme::Nord => "Nord",
            ColorScheme::GruvboxDark => "Gruvbox Dark",
            ColorScheme::TokyoNight => "Tokyo Night",
            ColorScheme::MaterialDark => "Material Dark",
            ColorScheme::Cyberpunk => "Cyberpunk",
        }
    }
    
    /// Создать ColorScheme из имени
    pub fn from_name(name: &str) -> Option<ColorScheme> {
        match name {
            "GitHub Light" => Some(ColorScheme::GithubLight),
            "Solarized Light" => Some(ColorScheme::SolarizedLight),
            "Atom Light" => Some(ColorScheme::AtomLight),
            "Visual Studio Light" => Some(ColorScheme::VisualStudioLight),
            "Dracula" => Some(ColorScheme::Dracula),
            "Monokai" => Some(ColorScheme::Monokai),
            "One Dark" => Some(ColorScheme::OneDark),
            "Nord" => Some(ColorScheme::Nord),
            "Gruvbox Dark" => Some(ColorScheme::GruvboxDark),
            "Tokyo Night" => Some(ColorScheme::TokyoNight),
            "Material Dark" => Some(ColorScheme::MaterialDark),
            "Cyberpunk" => Some(ColorScheme::Cyberpunk),
            _ => None,
        }
    }
    
    /// Получить цвет фона интерфейса (в формате RGB)
    pub fn background_rgb(&self) -> (u8, u8, u8) {
        match self {
            // Светлые темы - светлый фон
            ColorScheme::GithubLight => (255, 255, 255),
            ColorScheme::SolarizedLight => (253, 246, 227),
            ColorScheme::AtomLight => (250, 250, 250),
            ColorScheme::VisualStudioLight => (245, 245, 245),
            // Темные темы - темный фон
            ColorScheme::Dracula => (40, 42, 54),
            ColorScheme::Monokai => (39, 40, 34),
            ColorScheme::OneDark => (40, 44, 52),
            ColorScheme::Nord => (46, 52, 64),
            ColorScheme::GruvboxDark => (40, 40, 40),
            ColorScheme::TokyoNight => (25, 26, 35),
            ColorScheme::MaterialDark => (33, 33, 33),
            ColorScheme::Cyberpunk => (10, 0, 20),
        }
    }
    
    /// Получить цвет для фона выделенного элемента (в формате RGB)
    pub fn highlight_bg_rgb(&self) -> (u8, u8, u8) {
        match self {
            // Светлые темы - темный выделенный фон
            ColorScheme::GithubLight => (225, 228, 232),
            ColorScheme::SolarizedLight => (238, 232, 213),
            ColorScheme::AtomLight => (225, 228, 232),
            ColorScheme::VisualStudioLight => (225, 230, 240),
            // Темные темы - светлый выделенный фон
            ColorScheme::Dracula => (68, 71, 90),
            ColorScheme::Monokai => (60, 60, 60),
            ColorScheme::OneDark => (60, 65, 75),
            ColorScheme::Nord => (67, 76, 94),
            ColorScheme::GruvboxDark => (60, 56, 54),
            ColorScheme::TokyoNight => (45, 47, 63),
            ColorScheme::MaterialDark => (50, 50, 50),
            ColorScheme::Cyberpunk => (70, 0, 100),
        }
    }
    
    /// Получить цвет для текста (в формате RGB)
    pub fn text_color_rgb(&self) -> (u8, u8, u8) {
        match self {
            // Светлые темы - темный текст
            ColorScheme::GithubLight => (36, 41, 46),
            ColorScheme::SolarizedLight => (101, 123, 131),
            ColorScheme::AtomLight => (33, 37, 43),
            ColorScheme::VisualStudioLight => (30, 30, 30),
            // Темные темы - светлый текст
            ColorScheme::Dracula => (248, 248, 242),
            ColorScheme::Monokai => (248, 248, 242),
            ColorScheme::OneDark => (171, 178, 191),
            ColorScheme::Nord => (216, 222, 233),
            ColorScheme::GruvboxDark => (235, 219, 178),
            ColorScheme::TokyoNight => (195, 200, 210),
            ColorScheme::MaterialDark => (220, 220, 220),
            ColorScheme::Cyberpunk => (255, 0, 255),
        }
    }
    
    /// Преобразовать RGB цвет в формат ratatui::style::Color
    pub fn rgb_to_ratatui_color(&self, rgb: (u8, u8, u8)) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(rgb.0, rgb.1, rgb.2)
    }
    
    /// Получить цвет фона интерфейса в формате ratatui
    pub fn background(&self) -> ratatui::style::Color {
        self.rgb_to_ratatui_color(self.background_rgb())
    }
    
    /// Получить цвет для фона выделенного элемента в формате ratatui
    pub fn highlight_bg(&self) -> ratatui::style::Color {
        self.rgb_to_ratatui_color(self.highlight_bg_rgb())
    }

    /// Получить цвет для фона выделенных элементов (для множественного выбора)
    pub fn selection_bg(&self) -> ratatui::style::Color {
        let rgb = match self {
            // Светлые темы
            ColorScheme::GithubLight => (200, 220, 240),
            ColorScheme::SolarizedLight => (220, 225, 200),
            ColorScheme::AtomLight => (210, 215, 220),
            ColorScheme::VisualStudioLight => (210, 215, 225),
            // Темные темы
            ColorScheme::Dracula => (56, 60, 74),
            ColorScheme::Monokai => (50, 50, 50),
            ColorScheme::OneDark => (50, 55, 65),
            ColorScheme::Nord => (59, 66, 82),
            ColorScheme::GruvboxDark => (50, 48, 47),
            ColorScheme::TokyoNight => (35, 37, 50),
            ColorScheme::MaterialDark => (40, 40, 40),
            ColorScheme::Cyberpunk => (50, 0, 80),
        };
        self.rgb_to_ratatui_color(rgb)
    }
    
    /// Получить цвет для текста в формате ratatui
    pub fn text_color(&self) -> ratatui::style::Color {
        self.rgb_to_ratatui_color(self.text_color_rgb())
    }
    
    pub fn all() -> Vec<ColorScheme> {
        vec![
            ColorScheme::GithubLight,
            ColorScheme::SolarizedLight,
            ColorScheme::AtomLight,
            ColorScheme::VisualStudioLight,
            ColorScheme::Dracula,
            ColorScheme::Monokai,
            ColorScheme::OneDark,
            ColorScheme::Nord,
            ColorScheme::GruvboxDark,
            ColorScheme::TokyoNight,
            ColorScheme::MaterialDark,
            ColorScheme::Cyberpunk,
        ]
    }
}

/// Принцип сортировки файлов
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Поле сортировки файлов
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum SortField {
    Name,
    Size,
    Modified,
}

/// Настройки отображения интерфейса
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DisplaySettings {
    /// Отображать левую панель
    pub show_left_pane: bool,
    /// Отображать вкладки
    pub show_tabs: bool,
    /// Отображать нижнюю панель
    pub show_footer: bool,
    /// Показывать скрытые файлы
    pub show_hidden_files: bool,
    /// Цветовая схема
    pub color_scheme: ColorScheme,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            show_left_pane: true,
            show_tabs: true,
            show_footer: true,
            show_hidden_files: false,
            color_scheme: ColorScheme::Dracula,
        }
    }
}

/// Настройки сортировки файлов
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SortSettings {
    /// Поле сортировки
    pub field: SortField,
    /// Порядок сортировки
    pub order: SortOrder,
    /// Разделять файлы и каталоги
    pub separate_dirs: bool,
}

impl Default for SortSettings {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            order: SortOrder::Ascending,
            separate_dirs: true,
        }
    }
}

/// Настройки предпросмотра
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PreviewSettings {
    /// Включить предпросмотр
    pub enabled: bool,
    /// Максимальный размер файла для предпросмотра (в байтах)
    pub max_preview_size: u64,
    /// Разрешение для предпросмотра изображений
    pub image_preview_resolution: (u32, u32),
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_preview_size: 1024 * 1024, // 1MB
            image_preview_resolution: (800, 600),
        }
    }
}

/// Настройки поведения приложения
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BehaviorSettings {
    /// Подтверждение удаления файлов
    pub confirm_delete: bool,
    /// Подтверждение перезаписи при копировании/перемещении
    pub confirm_overwrite: bool,
    /// Автоматическое обновление содержимого каталогов
    pub auto_refresh: bool,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            confirm_delete: true,
            confirm_overwrite: true,
            auto_refresh: true,
        }
    }
}

/// Основная структура настроек приложения
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    /// Настройки отображения
    pub display: DisplaySettings,
    /// Настройки сортировки
    pub sort: SortSettings,
    /// Настройки предпросмотра
    pub preview: PreviewSettings,
    /// Настройки поведения
    pub behavior: BehaviorSettings,
    /// Закладки
    pub bookmarks: HashMap<String, PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display: DisplaySettings::default(),
            sort: SortSettings::default(),
            preview: PreviewSettings::default(),
            behavior: BehaviorSettings::default(),
            bookmarks: HashMap::new(),
        }
    }
}

impl Settings {
    /// Создать новые настройки со значениями по умолчанию
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Применить настройки к состоянию приложения
    pub fn apply_to_app_state(&self, app_state: &mut crate::app_state::AppState) {
        app_state.show_hidden_files = self.display.show_hidden_files;
        
        // Применяем другие настройки по мере необходимости
        // ...
    }
}