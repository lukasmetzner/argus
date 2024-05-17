use std::{fs, io::Read, path::Path};

use anyhow::Result;

use crate::tasks::{Task, TaskContainer};

#[derive(Debug, Clone)]
pub struct Scroll {
    pub name: String,
    pub tasks: Vec<TaskContainer>,
}

pub fn parse_scroll(scroll_dir_path: &Path) -> Result<Scroll> {
    let mut file =
        fs::File::open(scroll_dir_path.join("main.yml")).expect("Could not parse scroll!");
    let mut str_buf = String::new();
    file.read_to_string(&mut str_buf)
        .expect("Could not read hosts.yaml!");
    let tasks: Vec<TaskContainer> = serde_yaml::from_str::<Vec<Task>>(&str_buf)
        .unwrap()
        .into_iter()
        .map(|task| TaskContainer {
            scroll_dir_path: scroll_dir_path.to_owned(),
            task,
        })
        .collect();

    let scroll_name = scroll_dir_path
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap(); // TODO: Check

    Ok(Scroll {
        name: scroll_name,
        tasks,
    })
}
