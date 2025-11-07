use std::io::Write;
mod error;

struct TokenizedOutput<'a>{
    command:&'a str,
    args:Vec<&'a str>
}
fn main(){
    
    
    loop {
        let mut input_line = String::from("");
        print!("~$ ");
        std::io::stdout().flush().expect("Failed to flush stdout"); // Flush stdout to ensure prompt is displayed

        std::io::stdin().read_line(&mut input_line) // Read a line from stdin into the `input` string
            .expect("Failed to read line");

        let trimmed_input = input_line.trim().to_string();
        let tokens = tokenize_input(&trimmed_input);
        match_expression(tokens)
    }
}

fn tokenize_input<'a>(input_line:&'a String)->TokenizedOutput<'a>{
    let string_split:Vec<&str> = input_line.split_whitespace().collect();
    if string_split.len() == 0{
        //return error of empty input
    }
    //As no argument case handled above, shouldn't get an error here
    let command = string_split.first().map(|s|*s).unwrap_or("No command passed!");
    let args = string_split[1..].to_vec();
    TokenizedOutput{
        command,
        args
    }
}

fn match_expression(tokens:TokenizedOutput){
    match tokens.command {
        "echo"=>{println!("{}",tokens.args.concat())},
        "exit"=>{
            println!("bye");
            std::process::exit(0);
        }
        _ =>{eprintln!("Command not found!")}
    }
}