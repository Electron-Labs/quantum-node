use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigData {
    pub user_circuit_vk_path: String
}

pub fn load_config_data() -> ConfigData {
    let config_contents_str = fs::read_to_string("config.yaml").expect("provide a valid path");
    let config_data: ConfigData = serde_yaml::from_str(&config_contents_str).unwrap();
    println!("Config loaded: {:?}", config_data);
    // info!("loaded config data: {:?}", config_data);
    config_data
}