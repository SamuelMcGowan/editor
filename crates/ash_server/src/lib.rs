use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Request {
    Quit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Response {
    Ok,
}

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "ash_editor")
}
