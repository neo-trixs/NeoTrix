use clap::Parser;
use nt_core_capability_tree::CapabilityCli;

fn main() {
    let cli = CapabilityCli::parse();
    if let Err(e) = cli.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}