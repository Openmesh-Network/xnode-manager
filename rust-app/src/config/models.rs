use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerConfiguration {
    pub flake: String,
    pub flake_lock: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerSettings {
    pub flake: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfigurationAction {
    Set {
        container: String,
        settings: Option<ContainerSettings>,
        update_inputs: Option<Vec<String>>,
    },
    Remove {
        container: String,
        backup: bool,
    },
}
