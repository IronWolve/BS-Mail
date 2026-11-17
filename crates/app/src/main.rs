//! Program entry: reads the arguments, shows the screen the way this run
//! asks for, and reports what went wrong when it cannot.

mod account_writer;
mod address_file;
mod accounts;
mod attach_read;
mod attachment_name;
mod attachment_store;
mod browser_login;
mod cli;
mod clock;
mod config_edit;
mod config_io;
mod data_lock;
mod errors;
mod fonts;
mod html_store;
mod identity;
mod launch_mode;
mod link_opener;
mod oauth_clients;
mod options_flow;
mod paths;
mod picture;
mod private_dirs;
mod pump;
mod report;
mod run;
mod scene;
mod screen;
mod setup_flow;
mod source;
mod state_file;
mod stats;
mod theme_list;
mod ticker;
mod tty_run;
mod unlock;
mod unused_settings;
mod window_run;

use std::process::ExitCode;

use launch_mode::{Launch, decide};

fn main() -> ExitCode {
    // The service name the keychain entries of the vault's key are kept under: the program's own.
    engine::vault::set_keychain_service(identity::slug());
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
        cli::Action::PrintConfig => print_config(&options),
        cli::Action::Draw => show(&options),
    }
}

/// Shows the screen whichever way this run decided on.
fn show(options: &cli::Options) -> ExitCode {
    let result = match decide(&options.mode) {
        Launch::Terminal if live_terminal(options) => tty_run::run(options),
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

/// Whether to run inside the terminal rather than draw once: when there is a
/// terminal to run in on both ends, and a one-shot draw was not asked for.
fn live_terminal(options: &cli::Options) -> bool {
    use std::io::IsTerminal;
    !options.print && std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Writes the settings in force as TOML on standard output; what was wrong
/// with the user's file goes to standard error.
fn print_config(options: &cli::Options) -> ExitCode {
    let dir = paths::data_dir(options.data_dir.as_ref());
    match config_io::load(dir.as_deref()) {
        Ok(loaded) => {
            for problem in &loaded.problems {
                report::print_error(problem);
            }
            // The keys in force, every action with its key, so the names can be seen and pasted.
            let mut shown = loaded.config.clone();
            shown.keys = ui::keys::KeyMap::new(&shown.keys).in_force();
            match toml::to_string_pretty(&shown) {
                Ok(text) => {
                    print!("{text}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    report::print_error(&format!("the settings could not be written out: {error}"));
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            report::print_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}
