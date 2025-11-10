use std::error::Error;


/// Classes of characters possible
#[derive(Debug,PartialEq,Eq)]
pub enum ShellTokens{
    Word(String), //Normal characters apart from reserved ones like $,|,&...
    Pipe, // | used to redirect output to another process
    RedirectAsInput, // > 
    RedirectAsOutput, // <
    DoubleQuotes, // "
    SingleQuotes, // '
    ParenthesesOpen, // (
    ParenthesesClose, // )
    Comment(String), // #
    // LogicalNot, // !
    Assignment, // =
    Escape, // \ makes next char literal
    Variable(String), // $
    ReservedWord(ReservedWord), // if,else,elif,! etc
    Whitespace, // Single 
}

#[derive(Debug,PartialEq,Eq)]
enum ReservedWord{
    // If,
    // Else,
    // Elif,
    // While,
    // For,
    // BracketOpen,
    // BracketClose,
    // Exclamation
}


pub fn tokenize_input_intermediate(input:&str)->Vec<ShellTokens>{

    let mut output_tokens:Vec<ShellTokens> = vec![];
    // Iterate character wise 
    let mut iterator = input.chars().peekable();

    while let Some(char) = iterator.peek(){
        match char {
                    '$' => {
                        output_tokens.push(ShellTokens::Variable(handle_variable(&mut iterator)));
                        continue;
                    },
                    '=' =>{
                        output_tokens.push(ShellTokens::Assignment);
                    },
                    '|' => {
                        output_tokens.push(ShellTokens::Pipe);
                    },
                    '>' => {
                        output_tokens.push(ShellTokens::RedirectAsInput);
                    },
                    '<' => {
                        output_tokens.push(ShellTokens::RedirectAsOutput);
                    },
                    ' ' => {
                        output_tokens.push(ShellTokens::Whitespace);
                    },
                    '\'' => {
                        output_tokens.push(ShellTokens::SingleQuotes);
                    },
                    char if char.is_alphanumeric() => {
                        output_tokens.push(ShellTokens::Word(handle_unreserved_chars(&mut iterator)));
                        continue;
                    },
                    '(' => {
                        output_tokens.push(ShellTokens::ParenthesesOpen);
                    },
                    ')' => {
                        output_tokens.push(ShellTokens::ParenthesesClose);
                    },
                    '\"' => {
                        output_tokens.push(ShellTokens::DoubleQuotes);
                    },
                    '#' => {
                        output_tokens.push(ShellTokens::Comment(handle_comment_line(&mut iterator)));
                        continue;
                    }
                    _ => {
                        eprintln!("Invalid!");
                    }

                }
        iterator.next();
    
    }


    output_tokens
}

fn handle_variable(iter: &mut std::iter::Peekable<std::str::Chars>)->String{
    let mut var_name = String::from("");
    while let Some(char) = iter.peek(){
        match char{
            char if char.is_alphanumeric() => {
                var_name+=&char.to_string();
            },
            ' ' => {
                println!("{}",char);
                return var_name;
            },
            _ => {
                eprintln!("Invalid character found!")
            }

        }
        iter.next();
    }
    var_name
}

fn handle_unreserved_chars(iter: &mut std::iter::Peekable<std::str::Chars>)->String{
    let mut word = String::from("");
        while let Some(char) = iter.peek(){
        match char{
            char if char.is_alphanumeric() => {
                word.push(*char);
            },
            
            _ => {
                return word;
            }

        }
        iter.next();
    }
    word
}

fn handle_comment_line(iter: &mut std::iter::Peekable<std::str::Chars>)->String{
    let mut commented_line = String::from("");
    // while let Some(char) = iter.peek(){

    // }

    commented_line
}

#[cfg(test)]

mod tests{
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let tokens = tokenize_input_intermediate("echo hello $abc (d) | grep a");
        assert_eq!(
            tokens,
            vec![
                ShellTokens::Word("echo".into()),
                ShellTokens::Whitespace,
                ShellTokens::Word("hello".into()),
                ShellTokens::Whitespace,
                ShellTokens::Variable(String::from("abc")),
                ShellTokens::Whitespace,
                ShellTokens::ParenthesesOpen,
                ShellTokens::Word(String::from("d")),
                ShellTokens::ParenthesesClose,
                ShellTokens::Whitespace,
                ShellTokens::Pipe,
                ShellTokens::Whitespace,
                ShellTokens::Word(String::from("grep")),
                ShellTokens::Whitespace,
                ShellTokens::Word(String::from("a")),
            ]
        );
    }
}