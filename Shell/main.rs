use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::process::Command;
use std::path::Path;
use std::env;
use std::process::Stdio;
use std::process::Child;


fn main() {
    loop {
        // use the '>' character as the prompt
        // need to explicity flush this to ensure it prints before read_line
        
        let current_dir = env::current_dir().unwrap();
        print!("{}$ ", current_dir.display());
        stdout().flush().ok();

        let mut temp_input = String::new();
        stdin().read_line(&mut temp_input).unwrap();
       
        let mut input = String::new();
        while let Some(last_char) = temp_input.trim().chars().last(){
            input.push_str(&temp_input.trim());
            if last_char != '^'{
                break;
            }
            //Remove ^ from input
            input.pop();
            print!("> ");
            // Handle new line
            stdout().flush().ok();
            temp_input.clear();
            stdin().read_line(&mut temp_input).unwrap();
        }

        //must be peekable so we know when we are on the last command
        let mut commands = input.trim().split("|").peekable();
        let mut previous_command = None;
        
        while let Some(command) = commands.next() {
       
            //everyting after the first whitespace character
            //  is interpreted as args to the command
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    //default to '/' as new directory if one was not provided
                    let new_dir = args.peekable().peek()
                        .map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                },
                "exit" => return,
                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );
                    let stdout = if commands.peek().is_some() {
                        //there is another command piped behund this one
                        //prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        //there are no more commands piped behind this one
                        //send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            println!("{}", e);
                        },
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            //blocj until the final command has finished
            let _ = final_command.wait();
        }

    }
}
