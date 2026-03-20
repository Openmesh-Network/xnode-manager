use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FlakeQuery {
    pub flake: String,
}

#[derive(Serialize, Deserialize)]
pub struct EvalQuery {
    pub statement: String,
}
