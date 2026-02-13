//! A simple shell implementation in Rust
//! 
//! This library provides the core functionality for a command-line shell,
//! including command parsing, redirection handling, and command execution.

pub mod command;
pub mod redirect;
pub mod executor;

// Re-export commonly used items for convenience
pub use command::{Command, BUILT_IN_COMMANDS};
pub use executor::execute;
pub use redirect::Redirect;
