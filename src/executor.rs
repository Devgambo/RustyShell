use std::env;
use std::process::{Command as ProcessCommand, Stdio};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use pathsearch::find_executable_in_path;

use crate::command::{Command, BUILT_IN_COMMANDS};
use crate::redirect::{write_output, create_empty_stderr_file, open_redirect_file, Redirect};

/// Execute a parsed command
pub fn execute(command: Command) -> bool {
    match command {
        Command::ExitCommand => return false,
        
        Command::PwdCommand { redirect, stderr_redirect } => {
            let output = format!("{}", env::current_dir().unwrap().display());
            write_output(&output, redirect);
            create_empty_stderr_file(stderr_redirect);
        }
        
        Command::CdCommand { path } => {
            execute_cd(&path);
        }
        
        Command::EchoCommand { display_string, redirect, stderr_redirect } => {
            write_output(&display_string, redirect);
            create_empty_stderr_file(stderr_redirect);
        }
        
        Command::TypeCommand { command_name, redirect, stderr_redirect } => {
            let output = get_type_output(&command_name);
            write_output(&output, redirect);
            create_empty_stderr_file(stderr_redirect);
        }
        
        Command::ExternalCommand { program, args, redirect, stderr_redirect } => {
            execute_external(&program, &args, redirect, stderr_redirect);
        }
        
        Command::CommandNotFound { input } => {
            println!("{}: command not found", input);
        }
    }
    true // continue the shell loop
}

/// Execute the cd command
fn execute_cd(path: &str) {
    let path = if path.starts_with("~") {
        if let Ok(home) = env::var("HOME") {
            path.replacen("~", &home, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };
    
    if let Err(_) = env::set_current_dir(&path) {
        println!("cd: {}: No such file or directory", path);
    }
}

/// Get the output for the type command
fn get_type_output(command_name: &str) -> String {
    if BUILT_IN_COMMANDS.contains(&command_name) {
        format!("{} is a shell builtin", command_name)
    } else {
        if let Some(path) = find_executable_in_path(command_name) {
            format!("{} is {}", command_name, path.display())
        } else {
            format!("{}: not found", command_name)
        }
    }
}

/// Execute an external program
fn execute_external(program: &str, args: &[String], redirect: Redirect, stderr_redirect: Redirect) {
    if let Some(executable_path) = find_executable_in_path(program) {
        #[cfg(unix)]
        let mut cmd = ProcessCommand::new(&executable_path);
        #[cfg(unix)]
        cmd.arg0(program).args(args);
        
        #[cfg(windows)]
        let mut cmd = ProcessCommand::new(&executable_path);
        #[cfg(windows)]
        cmd.args(args);
        
        // Handle stdout redirection
        let stdout_configured = configure_stdout(&mut cmd, redirect);
        
        // Handle stderr redirection
        let stderr_configured = configure_stderr(&mut cmd, stderr_redirect);
        
        if stdout_configured && stderr_configured {
            match cmd.status() {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("{}: execution failed: {}", program, e);
                }
            }
        }
    } else {
        println!("{}: command not found", program);
    }
}

/// Configure stdout for a command (with optional redirection)
fn configure_stdout(cmd: &mut ProcessCommand, redirect: Redirect) -> bool {
    if let Some((file_path, is_append)) = redirect {
        match open_redirect_file(&file_path, is_append) {
            Ok(file) => {
                cmd.stdout(Stdio::from(file));
                true
            }
            Err(e) => {
                eprintln!("Failed to create file {}: {}", file_path, e);
                false
            }
        }
    } else {
        cmd.stdout(Stdio::inherit());
        true
    }
}

/// Configure stderr for a command (with optional redirection)
fn configure_stderr(cmd: &mut ProcessCommand, stderr_redirect: Redirect) -> bool {
    if let Some((file_path, is_append)) = stderr_redirect {
        match open_redirect_file(&file_path, is_append) {
            Ok(file) => {
                cmd.stderr(Stdio::from(file));
                true
            }
            Err(e) => {
                eprintln!("Failed to create stderr file {}: {}", file_path, e);
                false
            }
        }
    } else {
        cmd.stderr(Stdio::inherit());
        true
    }
}
