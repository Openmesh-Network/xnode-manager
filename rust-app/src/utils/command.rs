use std::{
    fmt::Display,
    fs::{create_dir_all, write, File},
    io::Error,
    process::Command,
    time::SystemTime,
};

use log::warn;

use crate::{
    request::models::RequestId,
    utils::{env::commandstream, output::Output},
};

pub enum CommandOutputError {
    OutputError { output: Vec<u8> },
    CommandError { e: Error },
}
impl Display for CommandOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                CommandOutputError::OutputError { output } => {
                    match output.clone().into() {
                        Output::UTF8 { output } => output.to_string(),
                        Output::Bytes { output } => {
                            format!("Non UTF8 output: {:?}", output)
                        }
                    }
                }
                CommandOutputError::CommandError { e } => e.to_string(),
            }
        )
    }
}
pub enum CommandExecutionMode {
    Simple,
    Stream { request_id: RequestId },
}
pub type CommandResult = Result<Vec<u8>, CommandOutputError>;
pub fn execute_command(mut command: Command, mode: CommandExecutionMode) -> CommandResult {
    log::info!("Executing command: {:?}", command);

    let mut on_result: Box<dyn Fn(&CommandResult)> = Box::new(|_| {});
    if let CommandExecutionMode::Stream { request_id } = &mode {
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

    let output = match if matches!(&mode, CommandExecutionMode::Simple) {
        command.output()
    } else {
        command.spawn().and_then(|c| c.wait_with_output())
    } {
        Ok(output_raw) => {
            if !output_raw.status.success() {
                return Err(CommandOutputError::OutputError {
                    output: output_raw.stderr,
                });
            }

            Ok(output_raw.stdout)
        }
        Err(e) => Err(CommandOutputError::CommandError { e }),
    };
    on_result(&output);

    output
}
