//! Program entry: reads the arguments, shows the screen the way this run
//! asks for, and reports what went wrong when it cannot.

mod cli;
mod clock;
mod errors;
mod fonts;
mod identity;
mod launch_mode;
mod paths;
mod picture;
mod pump;
mod report;
mod run;
mod scene;
mod screen;
mod source;
mod state_file;
mod stats;
mod ticker;
mod window_run;

use std::process::ExitCode;

use launch_mode::{Launch, decide};

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
        cli::Action::Draw => show(&options),
    }
}

/// Shows the screen whichever way this run decided on.
fn show(options: &cli::Options) -> ExitCode {
    let result = match decide(&options.mode) {
        Launch::Terminal => run::draw(options),
        Launch::Window => window_run::run(options),
        Launch::Picture(path) => picture::render(options, &path),
    };
    match result {
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
