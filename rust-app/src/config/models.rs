use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerConfiguration {
    pub flake: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfigurationAction {
    Set {
        container: String,
        config: ContainerConfiguration,
    },
    Remove {
        container: String,
        backup: bool,
    },
    Update {
        container: String,
        inputs: Vec<String>,
    },
}
