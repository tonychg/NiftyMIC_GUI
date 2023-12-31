use log::debug;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Output, Stdio},
};

use crate::config::Config;
use crate::niftymic::*;

pub fn print_output(output: &Output) {
    let s = std::str::from_utf8(&output.stdout).unwrap();
    for line in s.split("\n") {
        if !line.is_empty() {
            debug!("{}", line);
        }
    }
}

pub fn spawn_command(binary: &str, args: &Vec<String>, current_dir: Option<&str>) -> Result<()> {
    let current_dir = match current_dir {
        Some(directory) => directory,
        None => ".",
    };
    debug!("{} {}", binary, args.join(" "));
    let mut cmd = Command::new(binary)
        .args(args)
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            if let Ok(log_line) = line {
                debug!("{}", log_line);
            }
        }
    }
    if let Ok(exit_status) = cmd.wait() {
        if exit_status.success() {
            return Ok(());
        }
    }
    Err(Error::CommandFailed("Mask generation failed".to_string()))
}

pub struct DockerWrapper {
    pub executable: String,
    pub working_directory: String,
    pub image: String,
}

impl DockerWrapper {
    pub fn new(executable: &str, working_directory: &str, image: &str) -> DockerWrapper {
        DockerWrapper {
            executable: executable.to_string(),
            working_directory: working_directory.to_string(),
            image: image.to_string(),
        }
    }

    pub fn from_config(config: &Config) -> DockerWrapper {
        DockerWrapper::new(
            &config.executables.docker,
            &config.docker.working_directory,
            &config.docker.image,
        )
    }

    pub fn get_run_command_args(&self, working_directory: &str) -> Vec<String> {
        let mut args = Vec::new();
        args.push("run".to_string());
        args.push("--rm".to_string());
        args.push("-v".to_string());
        args.push(format!("{}:{}", working_directory, self.working_directory));
        args.push(self.image.to_string());
        args
    }

    pub fn run(&self, command: &str, args: &Vec<String>, working_directory: &str) -> Result<()> {
        let mut command_line = self.get_run_command_args(working_directory).clone();
        command_line.push(command.to_string());
        command_line.append(&mut args.clone());
        spawn_command(&self.executable, &command_line, None)
    }
}
