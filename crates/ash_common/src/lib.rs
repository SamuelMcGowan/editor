use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Quit,
}

#[derive(Serialize, Deserialize)]
pub enum Response {}
