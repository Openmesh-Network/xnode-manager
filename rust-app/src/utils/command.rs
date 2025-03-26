use std::{io::Error, process::Command, string::FromUtf8Error};

pub enum CommandOutput {
    OutputRaw(Vec<u8>, FromUtf8Error),
    Output(String),
}
pub enum CommandOutputError {
    OutputErrorRaw(Vec<u8>, FromUtf8Error),
    OutputError(String),
    CommandError(Error),
}
pub fn execute_command(mut command: Command) -> Result<CommandOutput, CommandOutputError> {
    log::info!("Executing command: {:?}", command);

    match command.output() {
        Ok(output_raw) => {
            if !output_raw.status.success() {
                let output = output_raw.stderr;
                return Err(String::from_utf8(output.clone())
                    .map(CommandOutputError::OutputError)
                    .unwrap_or_else(|e| CommandOutputError::OutputErrorRaw(output, e)));
            }

            let output = output_raw.stdout;
            Ok(String::from_utf8(output.clone())
                .map(CommandOutput::Output)
                .unwrap_or_else(|e| CommandOutput::OutputRaw(output, e)))
        }
        Err(e) => Err(CommandOutputError::CommandError(e)),
    }
}
