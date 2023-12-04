use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Request {
    Quit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Response {
    Ok,
}
