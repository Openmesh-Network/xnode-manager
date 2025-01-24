use std::{
    io::{Error, Result},
    process::Output,
};

pub enum CommandOutputError {
    OutputErrorRaw(Vec<u8>),
    OutputError(String),
    CommandError(Error),
}
pub fn command_output_errors(output: Result<Output>) -> Option<CommandOutputError> {
    match output {
        Ok(output_raw) => {
            let output = output_raw.stderr;
            if !output_raw.status.success() {
                return Some(
                    String::from_utf8(output.clone())
                        .map(|s| CommandOutputError::OutputError(s))
                        .unwrap_or(CommandOutputError::OutputErrorRaw(output)),
                );
            }
        }
        Err(e) => {
            return Some(CommandOutputError::CommandError(e));
        }
    }

    None
}
