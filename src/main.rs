use std::{fs, io::{stdin, stdout, Write}, process::exit};

use another_interpreted_language::{errors::ErrorList, evaluator::{object::ObjectValue, Evaluator}, extract_type, lexer::{token::Tokens, Lexer}, parser::{ast::Node, Parser, ParserErrors, TokenMismatch}, utils::colors::{BLUE, BOLD, GREEN, MAGENTA, RED, RESET, UNDERLINE}};

pub const NAME: &str = "YAIPL";
pub const NAME_LONG: &str = "Yet Another Interpreted Programming Language";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // No input file provided, run REPL
        repl();
        exit(1);
    }

    // Input file provided
    let file_path = &args[1];
    let cwd = std::env::current_dir().expect("Could not get current directory");
    let absolute_path = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(err) => {
            println!("Could not get absolute path: {}", err);
            exit(1)
        }
    };

    let pretty_path = match absolute_path.strip_prefix(&cwd) {
        Ok(path) => path.display().to_string(),
        Err(_) => absolute_path.display().to_string()
    };

    if let Err(err) = parse_file(file_path) {
        handle_errors(err, Some(pretty_path));
    }
}

pub fn repl() {
    println!("{}{}{} - {}REPL Mode{}", 
        format!("{}{}{}", GREEN, BOLD, UNDERLINE),
        NAME,
        RESET,
        format!("{}{}", BLUE, BOLD), 
        RESET
    );
    
    let stdin = stdin();
    let mut buf = String::new();

    loop {
        print!("\n{}{}>>>{} ", BOLD, BLUE, RESET);
        let _ = stdout().flush();
        let _ = stdin.read_line(&mut buf);

        let (_, _, result) = match interpret(buf.to_owned()) {
            Ok(res) => res,
            Err(err) => {
                handle_errors(err, None);
                buf.clear();
                continue;
            }
        };

        println!("{:?}", result);
    }
}

fn interpret(input: String) -> Result<(Tokens, Node, ObjectValue), Box<dyn ErrorList>> {
    let now = std::time::Instant::now();

    let mut lexer = Lexer::from(&input);
    let tokens = lexer.tokenize()?;
    let lexer_elapsed = now.elapsed().as_micros();

    let mut parser = Parser::from(&tokens);
    let ast = parser.parse()?;
    let parser_elapsed = now.elapsed().as_micros() - lexer_elapsed;

    let mut evaluator = Evaluator::new(ast.to_owned());
    let result = evaluator.eval()?;
    let eval_elapsed = now.elapsed().as_micros() - parser_elapsed;

    println!("
Lexer: {}{}ms
Parser: {}{}ms
Evaluator: {}{}ms
Total: {}{}ms
    ", 
        format!("{}{}{:.2}", GREEN, BOLD, lexer_elapsed as f32 / 1000.0),
        RESET,
        format!("{}{}{:.2}", GREEN, BOLD, parser_elapsed as f32 / 1000.0),
        RESET,
        format!("{}{}{:.2}", GREEN, BOLD, eval_elapsed as f32 / 1000.0),
        RESET,
        format!("{}{}{}", GREEN, BOLD, (lexer_elapsed + parser_elapsed + eval_elapsed) as f32 / 1000.0),
        RESET
    );

    Ok((tokens.to_owned(), ast, result))
}

pub fn parse_file(path: &String) -> Result<(), Box<dyn ErrorList>> {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            println!("Could not read file: {}", err);
            exit(1)
        } 
    };

    let (_, _, result) = interpret(content)?;

    if result != ObjectValue::Void {
        println!("{:?}", result);
    }

    Ok(())
}

fn handle_errors(err: Box<dyn ErrorList>, path: Option<String>) {
    extract_type!(err, ParserErrors, TokenMismatch, (mismatch) => {
        let path = match path {
            Some(path) => format!("{}:{}:{}", path, mismatch.position.line, mismatch.position.col),
            None => format!("{}:{}", mismatch.position.line, mismatch.position.col)
        };

        println!("{}{}{} error{} at '{}{}{}'", RED, BOLD, mismatch.get_name(), RESET, BLUE, path, RESET);

        print!("->{}{} ", MAGENTA, BOLD);
        if mismatch.expected.len() > 1 {
            println!("Expected tokens of type {:?} but found '{:?}'", mismatch.expected, mismatch.found);
        } else {
            println!("Expected token of type '{:?}' but found '{:?}'", mismatch.expected[0], mismatch.found);
        }
        print!("{}", RESET);

        return;
    });

    println!("{:#?}", err);
}

