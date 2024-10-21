use std::env::args;
use std::fs;
use std::fs::{DirEntry, OpenOptions};
use std::io::{BufRead, BufReader};
use clap::Parser;

/// Extract and filter moodle submissions based
/// on input lists
#[derive(Parser)]
struct Args {
    /// Dir of the filter lists
    #[arg(short, long)]
    filter: String,
    /// Where to find the input zip file
    #[arg(short, long)]
    source: String,
    /// Where to put the result
    #[arg(short, long)]
    target: String,
    /// Optional: Where to put the temp file (default = tmp)
    #[arg(long)]
    tmp_dir_name: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    init(&args);
    
    let filters = fs::read_dir(&args.filter)
        .expect(&format!("Unable fo find {}", args.filter));
    
    for filter in filters {
        let filter_dir = filter.unwrap();
        let filter_list = mk_filter_list(filter_dir);
    }
    
    cleanup(&args);
}

fn init(args: &Args) {}

fn mk_filter_list(filter_path: DirEntry) -> Vec<String> {
    let file = OpenOptions::new()
        .read(true)
        .open(filter_path.path())
        .expect(&format!("Unable to open {:?}", filter_path.path()));
    let reader = BufReader::new(file);
    
    reader.lines()
        .map(|l| l.unwrap())
        .filter(|l| l.starts_with("//"))
        .collect()
}

fn cleanup(args: &Args) {}
