use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let input = match (arguments.next(), arguments.next()) {
        (Some(input), None) => input,
        _ => {
            eprintln!("Usage: rumen <fumen-v115-or-url>");
            return ExitCode::from(2);
        }
    };
    match rumen::parse(&input) {
        Ok(fumen) => {
            println!("{fumen:#?}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
