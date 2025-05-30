use std::{
    fmt::Display,
    fs::{create_dir_all, write, File},
    io::Error,
    process::Command,
    string::FromUtf8Error,
    time::SystemTime,
};

use log::warn;

use crate::{request::models::RequestId, utils::env::commandstream};

pub enum CommandOutput {
    OutputRaw { output: Vec<u8>, e: FromUtf8Error },
    Output { output: String },
}
pub enum CommandOutputError {
    OutputErrorRaw { output: Vec<u8>, e: FromUtf8Error },
    OutputError { output: String },
    CommandError { e: Error },
}
impl Display for CommandOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                CommandOutputError::OutputErrorRaw { output, e } => {
                    format!("Output could not be decoded: {}. Output: {:?}", e, output)
                }
                CommandOutputError::OutputError { output } => output.to_string(),
                CommandOutputError::CommandError { e } => e.to_string(),
            }
        )
    }
}
pub enum CommandExecutionMode {
    Simple,
    Stream { request_id: RequestId },
}
pub type CommandResult = Result<CommandOutput, CommandOutputError>;
pub fn execute_command(mut command: Command, mode: CommandExecutionMode) -> CommandResult {
    log::info!("Executing command: {:?}", command);

    let mut on_result: Box<dyn Fn(&CommandResult)> = Box::new(|_| {});
    if let CommandExecutionMode::Stream { request_id } = mode {
        let start = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let path = commandstream()
            .join(request_id.to_string())
            .join(start.to_string());
        if let Err(e) = create_dir_all(&path) {
            warn!(
                "Could not create command execution dir {}: {}",
                path.display(),
                e
            );
        }
        if let Err(e) = write(path.join("command"), format!("{:?}", command)) {
            warn!(
                "Could not write command execution command file {}: {}",
                path.display(),
                e
            );
        }
        if let Ok(stdout) = File::create(path.join("stdout")) {
            command.stdout(stdout);
        }
        if let Ok(stderr) = File::create(path.join("stderr")) {
            command.stderr(stderr);
        }
        on_result = Box::new(move |result| {
            if let Err(e) = write(
                path.join("result"),
                match result {
                    Ok(_) => "0",
                    Err(_) => "1",
                },
            ) {
                warn!(
                    "Could not write command execution result file {}: {}",
                    path.display(),
                    e
                );
            }
        })
    }

    let output = match command.spawn().and_then(|c| c.wait_with_output()) {
        Ok(output_raw) => {
            if !output_raw.status.success() {
                let output = output_raw.stderr;
                return Err(String::from_utf8(output.clone())
                    .map(|output| CommandOutputError::OutputError { output })
                    .unwrap_or_else(|e| CommandOutputError::OutputErrorRaw { output, e }));
            }

            let output = output_raw.stdout;
            Ok(String::from_utf8(output.clone())
                .map(|output| CommandOutput::Output { output })
                .unwrap_or_else(|e| CommandOutput::OutputRaw { output, e }))
        }
        Err(e) => Err(CommandOutputError::CommandError { e }),
    };
    on_result(&output);

    output
}
