use std::env::args;
use clap::Parser;

/// Extract and filter moodle submissions based
/// on input lists
#[derive(Parser)]
struct Args {
    /// Dir of the filter lists
    #[arg(short, long)]
    dir: String,
}

fn main() {
    let filter_dir = Args::parse();
    
}
