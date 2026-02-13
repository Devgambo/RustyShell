use std::fs::{File, OpenOptions};
use std::io::Write;

/// Redirect type: Option<(file_path, is_append)>
/// - None: no redirection
/// - Some((path, false)): overwrite mode (>)
/// - Some((path, true)): append mode (>>)
pub type Redirect = Option<(String, bool)>;

/// Extract stdout and stderr redirections from input string.
/// Returns (command_without_redirects, stdout_redirect, stderr_redirect)
pub fn extract_redirection(input: &str) -> (String, Redirect, Redirect) {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut stdout_redirect = None;
    let mut stderr_redirect = None;
    let mut command_end = input.len();
    let mut positions_to_remove = Vec::new();
    
    let mut chars = input.chars().enumerate().peekable();
    
    while let Some((i, ch)) = chars.next() {
        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '>' if !in_single_quote && !in_double_quote => {
                // Check if it's >> (append mode)
                let is_append = if i + 1 < input.len() && input.chars().nth(i + 1) == Some('>') {
                    chars.next(); // consume the second >
                    true
                } else {
                    false
                };
                
                // Check what comes before >
                let prev_char = if i > 0 { input.chars().nth(i - 1) } else { None };
                
                let (redirect_start, is_stderr) = if prev_char == Some('2') {
                    (i - 1, true)
                } else if prev_char == Some('1') {
                    (i - 1, false)
                } else {
                    (i, false)
                };
                
                // Extract the file path after > or >>
                let after_redirect_start = if is_append { i + 2 } else { i + 1 };
                let after_redirect = &input[after_redirect_start..].trim_start();
                if let Some(file_path) = shlex::split(after_redirect).and_then(|parts| parts.first().cloned()) {
                    let file_path_start = after_redirect_start + (after_redirect.len() - after_redirect.trim_start().len());
                    let file_path_end = file_path_start + file_path.len();
                    
                    if is_stderr {
                        stderr_redirect = Some((file_path, is_append));
                    } else {
                        stdout_redirect = Some((file_path, is_append));
                    }
                    
                    positions_to_remove.push((redirect_start, file_path_end));
                    command_end = command_end.min(redirect_start);
                }
            }
            _ => {}
        }
    }
    
    if positions_to_remove.is_empty() {
        return (input.to_string(), None, None);
    }
    
    let command_part = input[..command_end].trim_end().to_string();
    (command_part, stdout_redirect, stderr_redirect)
}

/// Open a file for redirection (either create/overwrite or append)
pub fn open_redirect_file(file_path: &str, is_append: bool) -> std::io::Result<File> {
    if is_append {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
    } else {
        File::create(file_path)
    }
}

/// Write output to stdout or to a redirected file
pub fn write_output(output: &str, redirect: Redirect) {
    if let Some((file_path, is_append)) = redirect {
        match open_redirect_file(&file_path, is_append) {
            Ok(mut file) => {
                writeln!(file, "{}", output).unwrap();
            }
            Err(e) => {
                eprintln!("Failed to create file {}: {}", file_path, e);
            }
        }
    } else {
        println!("{}", output);
    }
}

/// Create an empty stderr redirect file (for commands that don't produce stderr)
pub fn create_empty_stderr_file(stderr_redirect: Redirect) {
    if let Some((file_path, is_append)) = stderr_redirect {
        let _ = open_redirect_file(&file_path, is_append);
    }
}
