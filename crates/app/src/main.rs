//! Program entry: reads the arguments, draws one screen and reports what
//! went wrong when it cannot.

mod cli;
mod identity;
mod paths;
mod report;
mod run;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = match cli::parse(&args) {
        Ok(options) => options,
        Err(message) => {
            report::print_error(&message);
            return ExitCode::FAILURE;
        }
    };
    match options.action {
        cli::Action::Help => {
            print!("{}", report::help_text());
            ExitCode::SUCCESS
        }
        cli::Action::Version => {
            println!("{}", report::version_line());
            ExitCode::SUCCESS
        }
        cli::Action::Draw => draw(&options),
    }
}

fn draw(options: &cli::Options) -> ExitCode {
    match run::draw(options) {
        Ok(stats) => {
            if options.stats {
                report::print_stats(&stats);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            report::print_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}
