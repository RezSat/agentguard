mod changes;
mod check;
mod eslint;

use std::env;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next();

    if let Some(unexpected) = args.next() {
        eprintln!("unexpected argument: {unexpected}");
        eprintln!("Run 'agentguard --help' to see available commands.");
        return ExitCode::from(2);
    }

    match command.as_deref() {
        Some("--help" | "-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") | Some("version") => {
            println!("agentguard {VERSION}");
            ExitCode::SUCCESS
        }
        Some("changes") => changes::run(),
        Some("check") => check::run(),
        Some("audit") => check::run_audit(),
        Some(command) => {
            eprintln!("unknown command: {command}");
            eprintln!("Run 'agentguard --help' to see available commands.");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "AgentGuard {VERSION}

USAGE:
    agentguard [COMMAND]

COMMANDS:
    changes          List staged, unstaged, and untracked Git files
    check            Lint changed Python and JavaScript/TypeScript files
    audit            Lint repository Python and JavaScript/TypeScript files
    version          Print version information

OPTIONS:
    -h, --help       Print this help
    -V, --version    Print version information"
    );
}
