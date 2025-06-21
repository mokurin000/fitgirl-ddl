use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
}
