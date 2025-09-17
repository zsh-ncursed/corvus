use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use corvus_core::app_state::AppState;
use corvus_core::settings::ColorScheme;

/// Enum для отслеживания текущего режима навигации в настройках
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsNavigationMode {
    Categories,
    Items,
}

#[derive(Debug)]
pub struct SettingsState {
    /// Выбранная категория настроек
    pub selected_category: usize,
    /// Выбранный элемент в категории
    pub selected_item: usize,
    /// Режим навигации
    pub navigation_mode: SettingsNavigationMode,
    /// Состояния для списков
    pub categories_state: ListState,
    pub items_state: ListState,
    /// Флаг для отображения списка горячих клавиш
    pub show_hotkeys_list: bool,
}

impl SettingsState {
    pub fn new() -> Self {
        let mut categories_state = ListState::default();
        categories_state.select(Some(0));
        
        let mut items_state = ListState::default();
        items_state.select(Some(0));
        
        Self {
            selected_category: 0,
            selected_item: 0,
            navigation_mode: SettingsNavigationMode::Categories,
            categories_state,
            items_state,
            show_hotkeys_list: false,
        }
    }
    
    pub fn categories() -> Vec<&'static str> {
        vec![
            "Интерфейс",
            "Цветовые схемы",
            "Сортировка",
            "Предпросмотр",
            "Поведение",
            "Закладки",
            "Поиск",
            "Горячие клавиши",
            "Плагины",
        ]
    }
    
    /// Обновить состояние элементов при смене категории
    pub fn update_items_state(&mut self) {
        self.items_state.select(Some(0));
        self.selected_item = 0;
    }
    
    /// Переместить выбор вниз
    pub fn move_down(&mut self) {
        let categories = Self::categories();
        if self.selected_category < categories.len() - 1 {
            self.selected_category += 1;
            self.categories_state.select(Some(self.selected_category));
            self.update_items_state();
        }
    }
    
    /// Переместить выбор вверх
    pub fn move_up(&mut self) {
        if self.selected_category > 0 {
            self.selected_category -= 1;
            self.categories_state.select(Some(self.selected_category));
            self.update_items_state();
        }
    }
    
    /// Переместить выбор элемента вниз
    pub fn move_item_down(&mut self, app_state: &AppState) {
        let item_count = self.get_current_category_item_count(app_state);
        if self.selected_item < item_count - 1 {
            self.selected_item += 1;
            self.items_state.select(Some(self.selected_item));
        }
    }
    
    /// Переместить выбор элемента вверх
    pub fn move_item_up(&mut self, app_state: &AppState) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.items_state.select(Some(self.selected_item));
        }
    }
    
    /// Получить количество элементов в текущей категории
    fn get_current_category_item_count(&self, app_state: &AppState) -> usize {
        match self.selected_category {
            0 => 5, // Интерфейс
            1 => ColorScheme::all().len(), // Цветовые схемы
            2 => 3, // Сортировка
            3 => 3, // Предпросмотр
            4 => 3, // Поведение
            5 => 1, // Закладки
            6 => 2, // Поиск
            7 => 1, // Горячие клавиши
            8 => app_state.plugins.len().max(1), // Плагины
            _ => 0,
        }
    }
    
    /// Переключить режим навигации
    pub fn toggle_navigation_mode(&mut self) {
        self.navigation_mode = match self.navigation_mode {
            SettingsNavigationMode::Categories => SettingsNavigationMode::Items,
            SettingsNavigationMode::Items => SettingsNavigationMode::Categories,
        };
    }
    
    /// Обработать выбор элемента
    pub fn select_item(&mut self, app_state: &mut AppState) {
        match self.selected_category {
            0 => {
                // Интерфейс
                match self.selected_item {
                    3 => {
                        // Показывать скрытые файлы
                        app_state.toggle_hidden_files();
                    }
                    _ => {}
                }
            }
            1 => {
                // Цветовые схемы
                let all_schemes = ColorScheme::all();
                if self.selected_item < all_schemes.len() {
                    let selected_scheme = all_schemes[self.selected_item].clone();
                    // Обновляем цветовую схему в конфигурации приложения
                    app_state.set_color_scheme(selected_scheme);
                }
            }
            8 => {
                // Плагины
                if let Some(plugin) = app_state.plugins.get_mut(self.selected_item) {
                    plugin.enabled = !plugin.enabled;
                }
            }
            _ => {}
        }
    }
}

// Global state for settings screen
static mut SETTINGS_STATE: Option<SettingsState> = None;

pub fn render_settings_screen(frame: &mut Frame, area: Rect, app_state: &mut AppState) {
    // Initialize or get the settings state
    unsafe {
        if SETTINGS_STATE.is_none() {
            SETTINGS_STATE = Some(SettingsState::new());
        }
    }
    
    let settings_state = unsafe { SETTINGS_STATE.as_mut().unwrap() };
    
    // Если показываем список горячих клавиш, отображаем его
    if settings_state.show_hotkeys_list {
        render_hotkeys_list(frame, area, &app_state.get_current_color_scheme());
        return;
    }
    
    // Получаем текущую цветовую схему
    let current_scheme = app_state.get_current_color_scheme();
    
    let block = Block::default()
        .title("Настройки приложения (↑/↓ - навигация, Tab - переключение, Enter - выбор, Esc - выход)")
        .borders(Borders::ALL)
        .style(Style::default()
            .fg(current_scheme.text_color())
            .bg(current_scheme.background()));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Разделяем экран на две части: категории и детали
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(inner_area);
    
    let categories_area = chunks[0];
    let details_area = chunks[1];
    
    // Рендерим категории
    render_categories(frame, categories_area, settings_state, &current_scheme);
    
    // Рендерим детали выбранной категории
    render_category_details(frame, details_area, settings_state, app_state, &current_scheme);
}

fn render_categories(frame: &mut Frame, area: Rect, settings_state: &mut SettingsState, color_scheme: &ColorScheme) {
    let categories = SettingsState::categories();
    let items: Vec<ListItem> = categories
        .iter()
        .map(|&category| ListItem::new(category)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title("Категории")
            .borders(Borders::ALL)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .highlight_style(Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color()));

    frame.render_stateful_widget(list, area, &mut settings_state.categories_state);
}

fn render_category_details(frame: &mut Frame, area: Rect, settings_state: &mut SettingsState, app_state: &AppState, color_scheme: &ColorScheme) {
    let items: Vec<ListItem> = match settings_state.selected_category {
        0 => {
            // Интерфейс
            vec![
                ListItem::new(format!(
                    "[{}] Отображать левую панель",
                    " " // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "[{}] Отображать вкладки",
                    " " // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "[{}] Отображать нижнюю панель",
                    " " // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "[{}] Показывать скрытые файлы",
                    if app_state.show_hidden_files { "•" } else { " " }
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "Цветовая схема: {}",
                    app_state.config.theme.color_scheme.as_deref().unwrap_or("Dracula")
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
            ]
        }
        1 => {
            // Цветовые схемы
            let all_schemes = ColorScheme::all();
            let current_scheme_name = app_state.config.theme.color_scheme.as_deref().unwrap_or("Dracula");
            all_schemes
                .iter()
                .map(|scheme| {
                    let marker = if scheme.name() == current_scheme_name {
                        "•"
                    } else {
                        " "
                    };
                    ListItem::new(format!("[{}] {}", marker, scheme.name()))
                        .style(Style::default()
                            .fg(color_scheme.text_color())
                            .bg(color_scheme.background()))
                })
                .collect()
        }
        2 => {
            // Сортировка
            vec![
                ListItem::new(format!(
                    "Поле сортировки: {}",
                    "По имени" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "Порядок: {}",
                    "По возрастанию" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "Разделение файлов и каталогов: {}",
                    "Да" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
            ]
        }
        3 => {
            // Предпросмотр
            vec![
                ListItem::new(format!(
                    "[{}] Включить предпросмотр",
                    "•" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "Максимальный размер файла: {} байт",
                    1024 * 1024 // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "Разрешение изображений: {}x{}",
                    800, 600 // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
            ]
        }
        4 => {
            // Поведение
            vec![
                ListItem::new(format!(
                    "[{}] Подтверждение удаления файлов",
                    "•" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "[{}] Подтверждение перезаписи",
                    "•" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
                ListItem::new(format!(
                    "[{}] Автоматическое обновление",
                    "•" // Пока не реализовано
                )).style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background())),
            ]
        }
        5 => {
            // Закладки
            vec![
                ListItem::new("Управление закладками...")
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background())),
            ]
        }
        6 => {
            // Поиск
            vec![
                ListItem::new("Поиск по содержимому файлов по умолчанию")
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background())),
                ListItem::new("Чувствительность к регистру")
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background())),
            ]
        }
        7 => {
            // Горячие клавиши
            vec![
                ListItem::new("Нажмите Enter для просмотра полного списка...")
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background())),
            ]
        }
        8 => {
            if app_state.plugins.is_empty() {
                vec![ListItem::new("No plugins found.").style(
                    Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background()),
                )]
            } else {
                app_state
                    .plugins
                    .iter()
                    .map(|plugin| {
                        let marker = if plugin.enabled { "•" } else { " " };
                        let text = format!("[{}] {}", marker, plugin.manifest.name);
                        ListItem::new(text).style(
                            Style::default()
                                .fg(color_scheme.text_color())
                                .bg(color_scheme.background()),
                        )
                    })
                    .collect()
            }
        }
        _ => vec![ListItem::new("Выберите категорию")
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background()))],
    };
    
    let list = List::new(items)
        .block(Block::default()
            .title("Настройки")
            .borders(Borders::ALL)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .highlight_style(Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color()));

    frame.render_stateful_widget(list, area, &mut settings_state.items_state);
}

/// Рендеринг полного списка горячих клавиш
fn render_hotkeys_list(frame: &mut Frame, area: Rect, color_scheme: &ColorScheme) {
    let hotkeys = get_all_hotkeys();
    let items: Vec<ListItem> = hotkeys
        .iter()
        .map(|(key, description)| {
            ListItem::new(format!("{:<15} - {}", key, description))
                .style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background()))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title("Горячие клавиши")
            .borders(Borders::ALL)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .highlight_style(Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color()));

    let mut state = ListState::default();
    frame.render_stateful_widget(list, area, &mut state);
}

/// Handle key events for the settings screen
/// Returns true if the settings screen should be closed
pub fn handle_settings_key(key_code: crossterm::event::KeyCode, app_state: &mut AppState) -> bool {
    unsafe {
        if let Some(settings_state) = SETTINGS_STATE.as_mut() {
            match key_code {
                crossterm::event::KeyCode::Char('q') => {
                    // Reset the settings state when exiting
                    SETTINGS_STATE = None;
                    app_state.input_mode = corvus_core::app_state::InputMode::Normal;
                    return true;
                }
                crossterm::event::KeyCode::Esc => {
                    // Если показываем список горячих клавиш, скрываем его
                    if settings_state.show_hotkeys_list {
                        settings_state.show_hotkeys_list = false;
                    } else {
                        // Reset the settings state when exiting
                        SETTINGS_STATE = None;
                                                app_state.input_mode = corvus_core::app_state::InputMode::Normal;
                        return true;
                    }
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    match settings_state.navigation_mode {
                        SettingsNavigationMode::Categories => {
                            settings_state.move_down();
                        }
                        SettingsNavigationMode::Items => {
                            settings_state.move_item_down(app_state);
                        }
                    }
                }
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    match settings_state.navigation_mode {
                        SettingsNavigationMode::Categories => {
                            settings_state.move_up();
                        }
                        SettingsNavigationMode::Items => {
                            settings_state.move_item_up(app_state);
                        }
                    }
                }
                crossterm::event::KeyCode::Char('l') => {
                    settings_state.navigation_mode = SettingsNavigationMode::Items;
                }
                crossterm::event::KeyCode::Char('h') => {
                    settings_state.navigation_mode = SettingsNavigationMode::Categories;
                }
                crossterm::event::KeyCode::Tab => {
                    settings_state.toggle_navigation_mode();
                }
                crossterm::event::KeyCode::Enter => {
                    // Если выбрана категория "Горячие клавиши", показываем полный список
                    if settings_state.selected_category == 7 {
                        settings_state.show_hotkeys_list = true;
                    } else {
                        settings_state.select_item(app_state);
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// Получить список всех горячих клавиш с описаниями
fn get_all_hotkeys() -> Vec<(&'static str, &'static str)> {
    vec![
        // Навигация
        ("j или ↓", "Переместить курсор вниз"),
        ("k или ↑", "Переместить курсор вверх"),
        ("h или ←", "Перейти в родительский каталог"),
        ("l или → или Enter", "Войти в каталог"),
        ("J (Shift+j)", "Переместить курсор вниз и выделить файл"),
        ("K (Shift+k)", "Переместить курсор вверх и выделить файл"),
        
        // Вкладки
        ("Ctrl+n", "Создать новую вкладку"),
        ("Ctrl+w", "Закрыть текущую вкладку"),
        ("Ctrl+Tab", "Перейти к следующей вкладке"),
        ("Ctrl+Shift+Tab", "Перейти к предыдущей вкладке"),
        ("Alt+1-9", "Перейти к вкладке по номеру"),
        ("Alt+t", "Показать/скрыть панель вкладок"),
        
        // Файловые операции
        ("y", "Копировать файл(ы)"),
        ("x", "Вырезать файл(ы)"),
        ("d", "Удалить файл(ы)"),
        ("p", "Вставить файл(ы)"),
        ("m", "Добавить текущий каталог в закладки"),
        ("r", "Переименовать файл"),
        ("n", "Открыть меню создания (файл/каталог)"),
        ("  n+f", "Создать новый файл"),
        ("  n+d", "Создать новый каталог"),
        ("i", "Показать информацию о файле"),
        ("u", "Отмонтировать диск (в панели дисков) или удалить закладку"),
        ("c+m", "Изменить права доступа (chmod)"),
        ("c+o", "Изменить владельца (chown)"),
        
        // Выделение
        ("Shift+Space", "Снять выделение с текущего файла"),
        ("Esc", "Отменить все выделения"),
        
        // Поиск
        ("/", "Открыть диалог поиска"),
        
        // Настройки
        ("F2", "Открыть настройки"),
        
        // Терминал
        ("Ctrl+`", "Показать/скрыть встроенный терминал"),
        
        // Скрытие файлов
        (".", "Показать/скрыть скрытые файлы"),
        
        // Выход
        ("q", "Выход из приложения"),
    ]
}