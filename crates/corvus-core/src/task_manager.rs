use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::path::PathBuf;
use tokio::sync::mpsc;
use io::fs_ops;

#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
}

#[derive(Debug, Clone)]
pub enum TaskKind {
    Copy { src: PathBuf, dest: PathBuf },
    Move { src: PathBuf, dest: PathBuf },
    Delete { path: PathBuf },
    CreateFile { path: PathBuf },
    CreateDirectory { path: PathBuf },
    Chmod { path: PathBuf, mode: u32 },
    Chown { path: PathBuf, owner: String },
    Unmount { path: PathBuf },
    Archive { paths: Vec<PathBuf>, dest: PathBuf, format: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress(f32), // Progress from 0.0 to 1.0
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub kind: TaskKind,
    pub status: TaskStatus,
    pub description: String,
}

impl Task {
    pub fn new(kind: TaskKind, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            status: TaskStatus::Pending,
            description,
        }
    }
}

use std::fmt;

pub struct TaskManager {
    tasks: Arc<Mutex<Vec<Task>>>,
    progress_rx: mpsc::Receiver<(Uuid, fs_ops::ProgressEvent)>,
    progress_tx: mpsc::Sender<(Uuid, fs_ops::ProgressEvent)>,
}

impl fmt::Debug for TaskManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskManager")
         .field("tasks", &self.tasks.lock().unwrap())
         .finish()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            progress_rx: rx,
            progress_tx: tx,
        }
    }

    pub fn add_task(&self, kind: TaskKind, description: String) {
        let task = Task::new(kind, description);
        self.tasks.lock().unwrap().push(task);
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().clone()
    }

    pub fn process_pending_tasks(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        for task in tasks.iter_mut() {
            if task.status == TaskStatus::Pending {
                task.status = TaskStatus::InProgress(0.0);

                let task_id = task.id;
                let kind = task.kind.clone();
                let progress_tx = self.progress_tx.clone();

                tokio::spawn(async move {
                    match kind {
                        TaskKind::Copy { src, dest } => {
                            fs_ops::copy_file_task(task_id, src, dest, progress_tx).await;
                        }
                        TaskKind::Move { src, dest } => {
                            fs_ops::move_item_task(task_id, src, dest, progress_tx).await;
                        }
                        TaskKind::Delete { path } => {
                            fs_ops::delete_item_task(task_id, path, progress_tx).await;
                        }
                        TaskKind::CreateFile { path } => {
                            fs_ops::create_file_task(task_id, path, progress_tx).await;
                        }
                        TaskKind::CreateDirectory { path } => {
                            fs_ops::create_directory_task(task_id, path, progress_tx).await;
                        }
                        TaskKind::Chmod { path, mode } => {
                            fs_ops::chmod_task(task_id, path, mode, progress_tx).await;
                        }
                        TaskKind::Chown { path, owner } => {
                            fs_ops::chown_task(task_id, path, owner, progress_tx).await;
                        }
                        TaskKind::Unmount { path } => {
                            fs_ops::unmount_task(task_id, path, progress_tx).await;
                        }
                        TaskKind::Archive { paths, dest, format } => {
                            fs_ops::archive_task(task_id, paths, dest, format, progress_tx).await;
                        }
                    }
                });
            }
        }
    }

    pub async fn wait_for_event(&mut self) -> bool {
        if let Some((task_id, event)) = self.progress_rx.recv().await {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                match event {
                    fs_ops::ProgressEvent::Completed => {
                        task.status = TaskStatus::Completed;
                        return true;
                    }
                    fs_ops::ProgressEvent::Error(e) => {
                        task.status = TaskStatus::Failed(e);
                    }
                    fs_ops::ProgressEvent::Update(p) => {
                        task.status = TaskStatus::InProgress(p)
                    }
                }
            }
        }
        false
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
