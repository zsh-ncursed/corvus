use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use corvus_core::app_state::AppState;
use std::time::Duration;
use tokio::time::interval;
use ui::tui::{self, Tui};

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("rtfm.log")?)
        .apply()?;
    Ok(())
}

struct App {
    app_state: AppState,
    tui: Tui,
}

impl App {
    fn new() -> Result<Self> {
        let app_state = AppState::new();
        let tui = Tui::new()?;
        Ok(Self {
            app_state,
            tui,
        })
    }

    async fn run(&mut self) -> Result<()> {
        self.tui.enter()?;
        let mut event_stream = EventStream::new();
        let mut mount_update_interval = interval(Duration::from_secs(5));

        'main: loop {
            self.app_state.task_manager.process_pending_tasks();

            if let Some(timer) = self.app_state.notification_timer {
                if timer.elapsed().as_secs() > 3 {
                    self.app_state.notification = None;
                    self.app_state.notification_timer = None;
                }
            }

            self.tui.terminal.draw(|frame| {
                ui::layout::render_main_layout(frame, &mut self.app_state);
            })?;

            tokio::select! {
                biased;
                _ = mount_update_interval.tick() => {
                    self.app_state.update_mounts();
                }
                maybe_event = event_stream.next() => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Event::Key(key) = event {
                            if key.kind == KeyEventKind::Press {
                                if !tui::handle_key_press(key, &mut self.app_state) {
                                    break 'main;
                                }
                            }
                        }
                    } else {
                        break 'main;
                    }
                }
                task_completed = self.app_state.task_manager.wait_for_event() => {
                    if task_completed {
                        // Проверяем, была ли завершена задача архивирования
                        let tasks = self.app_state.task_manager.get_tasks();
                        if let Some(last_task) = tasks.last() {
                            if matches!(last_task.kind, corvus_core::task_manager::TaskKind::Archive { .. }) {
                                if last_task.status == corvus_core::task_manager::TaskStatus::Completed {
                                    // Получаем имя архива из описания задачи
                                    let archive_name = last_task.description.split(" to ").nth(1)
                                        .map(|s| s.trim_matches('"'))
                                        .unwrap_or("archive");
                                    self.app_state.notification = Some(format!("Archive {} created successfully", archive_name));
                                    self.app_state.notification_timer = Some(std::time::Instant::now());
                                }
                            }
                        }
                        
                        let show_hidden = self.app_state.show_hidden_files;
                        self.app_state.get_active_tab_mut().update_entries(show_hidden);
                        self.app_state.update_mounts(); // Also update mounts after a task completes
                    }
                }
            }
        }
        self.tui.exit()?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().expect("Failed to set up logger");
    log::info!("Application starting up");

    match App::new() {
        Ok(mut app) => {
            if let Err(e) = app.run().await {
                eprintln!("Error: {:?}", e);
                // To ensure the terminal state is restored.
                if let Err(exit_err) = app.tui.exit() {
                    eprintln!("Failed to properly exit TUI mode: {:?}", exit_err);
                }
                std::process::exit(1);
            }
            
            // Сохраняем сессию при выходе
            if let Err(e) = corvus_core::session::save_session(&app.app_state) {
                log::error!("Failed to save session: {:?}", e);
            } else {
                log::info!("Session saved successfully");
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize application: {:?}", e);
            std::process::exit(1);
        }
    }

    log::info!("Application shutting down");
    Ok(())
}
