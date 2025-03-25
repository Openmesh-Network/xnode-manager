use std::{io::Error, process::Command};

pub enum CommandOutputError {
    OutputErrorRaw(Vec<u8>),
    OutputError(String),
    CommandError(Error),
}
pub fn execute_command(mut command: Command) -> Option<CommandOutputError> {
    log::info!("Executing command: {:?}", command);

    match command.output() {
        Ok(output_raw) => {
            let output = output_raw.stderr;
            if !output_raw.status.success() {
                return Some(
                    String::from_utf8(output.clone())
                        .map(CommandOutputError::OutputError)
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
