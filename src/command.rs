use pathsearch::find_executable_in_path;
use crate::redirect::{extract_redirection, Redirect};

pub const BUILT_IN_COMMANDS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

/// Represents all possible commands the shell can handle
pub enum Command {
    ExitCommand,
    PwdCommand { redirect: Redirect, stderr_redirect: Redirect },
    CdCommand { path: String },
    EchoCommand { display_string: String, redirect: Redirect, stderr_redirect: Redirect },
    TypeCommand { command_name: String, redirect: Redirect, stderr_redirect: Redirect },
    ExternalCommand { program: String, args: Vec<String>, redirect: Redirect, stderr_redirect: Redirect },
    CommandNotFound { input: String },
}

impl Command {
    /// Parse user input and return the appropriate Command variant
    pub fn from_input(input: &str) -> Self {
        let input = input.trim();
        if input.is_empty() {
            return Self::CommandNotFound { input: input.to_string() };
        }
        
        // Extract redirection before parsing
        let (input_without_redirect, redirect, stderr_redirect) = extract_redirection(input);
        
        // Parse with shlex for proper quote handling
        let parts = match shlex::split(&input_without_redirect) {
            Some(parts) => parts,
            None => return Self::CommandNotFound { input: input.to_string() },
        };
        
        if parts.is_empty() {
            return Self::CommandNotFound { input: input.to_string() };
        }
        
        let command_name = &parts[0];
        
        if command_name == "exit" {
            return Self::ExitCommand;
        };

        if command_name == "pwd" {
            return Self::PwdCommand { redirect, stderr_redirect };
        };
        
        if command_name == "cd" {
            if parts.len() > 1 {
                return Self::CdCommand {
                    path: parts[1].clone(),
                };
            }
        }
        
        if command_name == "echo" {
            if parts.len() > 1 {
                return Self::EchoCommand {
                    display_string: parts[1..].join(" "),
                    redirect,
                    stderr_redirect,
                };
            } else {
                return Self::EchoCommand {
                    display_string: String::new(),
                    redirect,
                    stderr_redirect,
                };
            }
        }
        
        if command_name == "type" {
            if parts.len() > 1 {
                return Self::TypeCommand {
                    command_name: parts[1].clone(),
                    redirect,
                    stderr_redirect,
                };
            }
        }
        
        // Check if it's an external command
        if let Some(_) = find_executable_in_path(command_name) {
            return Self::ExternalCommand {
                program: command_name.to_string(),
                args: parts[1..].iter().map(|s| s.to_string()).collect(),
                redirect,
                stderr_redirect,
            };
        }
        
        Self::CommandNotFound { input: input.to_string() }
    }
}
