use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerConfiguration {
    pub flake: String,
    pub flake_lock: String,
    pub network: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerSettings {
    pub flake: String,
    pub network: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfigurationAction {
    Set {
        container: String,
        settings: ContainerSettings,
        update_inputs: Option<Vec<String>>,
    },
    Remove {
        container: String,
    },
}
