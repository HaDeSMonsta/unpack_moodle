use clap::Parser;
use std::fs::{DirEntry, OpenOptions};
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::{fs, io};
use zip::ZipArchive;

/// Extract and filter moodle submissions based
/// on input lists
#[derive(Parser)]
struct Args {
    /// Dir of the filter lists,
    /// supports comments via //,
    /// (No support for multi line comments)
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

fn main() -> Result<()> {
    let (filter_dir, input_zip, target_dir, tmp_dir) = parse_args();

    init(&target_dir, &tmp_dir, &input_zip)?;

    let filters = fs::read_dir(&filter_dir)?;

    for filter in filters {
        let filter_dir = filter?;
        let filter_list = mk_filter_list(filter_dir);
        println!("{filter_list:?}");
        logic(&tmp_dir, &target_dir, filter_list)?;
    }

    #[cfg(not(debug_assertions))]
    cleanup(&tmp_dir);

    Ok(())
}

fn parse_args() -> (String, String, String, String) {
    let args = Args::parse();
    let tmp = args.tmp_dir_name.unwrap_or(String::from("tmp/"));
    (args.filter, args.source, args.target, tmp)
}

fn init<P, Q, R>(target_dir: P, tmp_dir: Q, input_zip: R)
    -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    R: AsRef<Path>,
{
    // Remove target dir, if exist
    // Error if dir does not exist, we don't care
    let _ = fs::remove_dir_all(&target_dir);

    // Remove tmp dir if exist
    // We know tmp_dir_name is some, but this is the only way to not move out of args
    let _ = fs::remove_dir_all(&tmp_dir);

    // TODO Unzip source file into temp dir
    let file = OpenOptions::new()
        .read(true)
        .open(&input_zip)?;

    let mut archive = ZipArchive::new(BufReader::new(file))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let out_path = tmp_dir.as_ref().join(file.enclosed_name().unwrap());

        if file.is_dir() {
            fs::create_dir_all(&out_path)?
        } else {
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut out_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&out_path)?;

            io::copy(&mut file, &mut out_file)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            use std::fs::{set_permissions, Permissions};

            if let Some(mode) = file.unix_mode() {
                set_permissions(&out_path, Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

fn mk_filter_list(filter_path: DirEntry) -> Vec<String> {
    let file = OpenOptions::new()
        .read(true)
        .open(filter_path.path())
        .expect(&format!("Unable to open {:?}", filter_path.path()));
    let reader = BufReader::new(file);

    reader.lines()
          .map(|l| l.unwrap())
          .filter(|l| !l.starts_with("//"))
          .map(|l| {
              if l.contains("//") {
                  l.split("//")
                   .take(1)
                   .map(|s| s.trim())
                   .collect::<String>()
              } else { String::from(l.trim()) }
          })
          .collect()
}

// CONSIDER Add Prog1Tools and .idea config for it
fn logic<P, Q>(source_dir: P, target_dir: Q, filter_list: Vec<String>)
    -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    // TODO Search in tmp dir for name in list, extract into out dir
    
    Ok(())
}

#[cfg(not(debug_assertions))]
fn cleanup<P: AsRef<Path>>(tmp_dir: P) {
    // TODO Remove temp dir
    let _ = fs::remove_dir_all(tmp_dir);
}
