use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use corvus_core::app_state::{AppState, TerminalState};
use std::time::Duration;
use tokio::time::interval;
use ui::tui::{self, Tui};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read};
use tokio::sync::mpsc;
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
    terminal_rx: mpsc::Receiver<String>,
}

impl App {
    fn new() -> Result<Self> {
        let mut app_state = AppState::new();
        let tui = Tui::new()?;

        let (terminal_tx, terminal_rx) = mpsc::channel(100);

        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 9,
            cols: 80,
            ..Default::default()
        })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        let cmd = CommandBuilder::new(shell);
        let _child = pair.slave.spawn_command(cmd)?;

        let mut pty_reader = pair.master.try_clone_reader()?;

        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                match pty_reader.read(&mut buf) {
                    Ok(count) => {
                        if count > 0 {
                            let s = String::from_utf8_lossy(&buf[..count]).to_string();
                            if terminal_tx.send(s).await.is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let terminal_state = TerminalState {
            pty_writer: pair.master,
            lines: Vec::new(),
        };
        app_state.terminal = Some(terminal_state);

        Ok(Self {
            app_state,
            tui,
            terminal_rx,
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
                Some(s) = self.terminal_rx.recv() => {
                    if let Some(terminal) = &mut self.app_state.terminal {
                        terminal.lines.extend(s.lines().map(String::from));
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
