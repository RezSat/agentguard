use std::env;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("--help" | "-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") | Some("version") => {
            println!("agentguard {VERSION}");
            ExitCode::SUCCESS
        }
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

Reliable verification for AI-generated software.

USAGE:
    agentguard [OPTIONS]

OPTIONS:
    -h, --help       Print this help
    -V, --version    Print version information

More commands will be added as the verification pipeline takes shape."
    );
}
