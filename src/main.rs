use lualint::rules;

use std::io::Write;
mod cli;

fn main() {
    let log_fmt = |buf: &mut env_logger::fmt::Formatter, record: &log::Record| {
        writeln!(
            buf,
            "{}:{} [{}] - {}",
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            // chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
            record.level(),
            record.args()
        )
    };
    env_logger::Builder::from_env(env_logger::Env::default().filter("LUALINT_LOG"))
        .format(log_fmt)
        .init();
    let args = <cli::Args as clap::Parser>::parse();
    log::trace!("CLI Args = {:?}", args);
    rules::init_all();
    if let Some(cmd) = &args.command {
        match cmd {
            cli::Commands::Run { filename, rules } => cli::handle_run_command(filename, rules),
            cli::Commands::Rules {} => cli::print_rules(),
        }
    }
}
