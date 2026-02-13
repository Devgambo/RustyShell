use std::io::{self, Write};
use codecrafters_shell::{Command, execute};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        
        let command = Command::from_input(&input);
        
        if !execute(command) {
            break; // Exit command was received
        } 
    }
}
