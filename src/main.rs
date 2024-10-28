use clap::Parser;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::{fs, io};
use zip::ZipArchive;

/// Extract and filter moodle submissions based
/// on input lists
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    /// Dir of the filter lists,
    /// supports comments via //,
    /// (No support for multi line comments)
    #[arg(short, long, default_value = "filter/")]
    filter: String,
    /// Where to find the input zip file
    #[arg(short, long, default_value = "submissions.zip")]
    source: String,
    /// Where to put the result
    #[arg(short, long, default_value = "out/")]
    target: String,
    /// Optional: Where to put the temp file
    #[arg(long, default_value = "tmp/")]
    tmp_dir_name: String,
}

fn main() -> Result<()> {
    let (filter_dir, input_zip, target_root_dir, tmp_dir) = parse_args();

    init(&target_root_dir, &tmp_dir, &input_zip)?;

    let filters = fs::read_dir(&filter_dir)?;

    // CONSIDER Ignore file
    for filter in filters {
        let filter_dir = filter?;
        let filter_list = mk_filter_list(filter_dir.path());
        let target_name = filter_dir.file_name();
        let target_stem = Path::new(&target_name).file_stem().unwrap();
        let target_path = Path::new(&target_root_dir).join(target_stem);
        logic(&tmp_dir, &target_path, filter_list)?;
    }

    cleanup(&tmp_dir, &target_root_dir)?;

    println!("Program is done, thank you for your patience");

    Ok(())
}

fn parse_args() -> (String, String, String, String) {
    let args = Args::parse();
    (args.filter, args.source, args.target, args.tmp_dir_name)
}

fn init<P, Q, R>(target_dir: P, tmp_dir: Q, input_zip: R)
    -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    R: AsRef<Path>,
{
    let _ = fs::remove_dir_all(&target_dir);

    let _ = fs::remove_dir_all(&tmp_dir);

    unzip_to(&input_zip, &tmp_dir)
}

fn unzip_to<P, Q>(zip: P, dest: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let file = OpenOptions::new()
        .read(true)
        .open(&zip)?;

    let mut archive = ZipArchive::new(BufReader::new(file))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let out_path = dest.as_ref().join(file.enclosed_name().unwrap());

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

fn mk_filter_list<P: AsRef<Path> + Debug>(filter_path: P) -> Vec<String> {
    let file = OpenOptions::new()
        .read(true)
        .open(filter_path.as_ref())
        .expect(&format!("Unable to open {:?}", filter_path));
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
          .filter(|l| !l.is_empty())
          .collect()
}

// CONSIDER Add Prog1Tools and .idea config for it
fn logic<P, Q>(source_dir: P, target_dir: Q, mut filter_list: Vec<String>)
    -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let source_dir = source_dir.as_ref();
    let target_dir = target_dir.as_ref();

    for entry in fs::read_dir(&source_dir)? {
        let entry = entry?;
        let path = entry.path();

        assert!(
            path.is_dir(),
            "Everything in in {:?} should be a dir, found: {:?}", source_dir, path,
        );

        let mut count = 0u8;

        for file_entry in fs::read_dir(&path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();

            assert_eq!(count, 0, "Expected to find exactly one file, found more: {:?}", file_path);
            count += 1;

            assert_eq!(
                Some(String::from("zip")),
                file_path.extension()
                    .and_then(|e| e.to_str())
                    .and_then(|e| Some(e.to_ascii_lowercase())),
                "Expected to find a zip file, found {:?}", file_path,
            );

            let zip_dir_name = path.file_name()
                                   .and_then(|f| f.to_str())
                                   .unwrap_or("");

            let mut name = None;
            for filter in &filter_list {
                if zip_dir_name.contains(filter) {
                    let target_subdir = target_dir.join(&filter);
                    fs::create_dir_all(&target_subdir)?;
                    unzip_to(&file_path, &target_subdir)?;
                    name = Some(filter.clone());
                    break;
                }
            }
            if let Some(name) = name {
                filter_list = filter_list.into_iter()
                                         .filter(|s| *s != name)
                                         .collect();
            }
        }
    }

    for name in filter_list {
        eprintln!("Unable to find {name}");
    }

    Ok(())
}

#[allow(unused_variables)]
fn cleanup<P: AsRef<Path>, Q: AsRef<Path>>(tmp_dir: P, out_dir: Q) -> Result<()> {
    // CONSIDER Remove __MACOSX, Remove .idea (vsc/jdk), lib, .iml && out, Move Name/T => Name

    for lab_dir in fs::read_dir(&out_dir)? {
        let lab_dir = lab_dir?.path();
        for dir in fs::read_dir(lab_dir)? {
            let dir = dir?.path();
            let _ = fs::remove_dir_all(dir.join("__MACOSX"));
        }
    }

    // Is double iteration efficient? No
    // Do I care? No (maybe)
    // CONSIDER Add this logic to the loops above
    for lab_dir in fs::read_dir(&out_dir)? {
        let lab_dir = lab_dir?.path();
        for name_dir in fs::read_dir(&lab_dir)? {
            let name_dir = name_dir?.path();
            let mut count = 0;
            for dir in fs::read_dir(&name_dir)? {
                let dir = dir?.path();
                assert_eq!(0, count, "Each dir should only have on subdir, found more: {dir:?}");
                count += 1;

                let _ = fs::remove_dir_all(dir.join("out"));
            }
        }
    }

    #[cfg(not(debug_assertions))]
    let _ = fs::remove_dir_all(tmp_dir);

    Ok(())
}

