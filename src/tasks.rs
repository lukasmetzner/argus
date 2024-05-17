use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use tracing::info;

use crate::modules::{bash::Bash, bash_script::BashScript, file_sync::FileSync};

#[derive(Debug, Clone)]
pub struct TaskContainer {
    pub scroll_dir_path: PathBuf,
    pub task: Task,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub task_exec: TaskExec,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum TaskExec {
    Bash(Bash),
    BashScript(BashScript),
    FileSync(FileSync),
}

impl TaskContainer {
    pub fn run(&self, sess: &mut Session) -> Result<i32> {
        info!("--- {} ---", self.task.name);
        match &self.task.task_exec {
            TaskExec::Bash(bash) => bash.exec(sess),
            TaskExec::BashScript(bash_script) => bash_script.exec(sess),
            TaskExec::FileSync(file) => file.exec(sess, &self.scroll_dir_path),
        }
    }
}
