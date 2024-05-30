use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Protocol {
    pub protocol_name:  String,
    pub auth_token: String
}