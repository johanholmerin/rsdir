use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use os_str_bytes::RawOsString;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io, result};
use tempfile::NamedTempFile;

const DEFAULT_DIR: &str = ".";
const DEFAULT_EDITOR: &str = "vi";
const EDITOR_ENV: &str = "EDITOR";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Verbosely display the actions taken by the program
    #[arg(short, long)]
    verbose: bool,

    /// Directories to edit. Defaults to current directory
    path: Vec<String>,
}

#[derive(Debug)]
struct PathInfo {
    name: PathBuf,
    is_dir: bool,
}

#[derive(Debug)]
struct InputRow {
    index: usize,
    name: PathBuf,
    is_dir: bool,
}

#[derive(Debug)]
struct OutputRow {
    index: usize,
    name: PathBuf,
}

fn get_path_args(paths: Vec<String>) -> Vec<PathBuf> {
    if paths.is_empty() {
        vec![PathBuf::from(DEFAULT_DIR)]
    } else {
        paths.iter().map(PathBuf::from).collect()
    }
}

fn read_dir(path: &Path) -> result::Result<Vec<PathInfo>, io::Error> {
    fs::read_dir(path)?
        .map(|res| {
            let entry = res?;
            Ok(PathInfo {
                name: entry.path(),
                is_dir: entry.file_type()?.is_dir(),
            })
        })
        .collect()
}

fn list_files(paths: Vec<PathBuf>) -> Result<Vec<InputRow>> {
    let mut entries = Vec::<PathInfo>::new();

    for path in paths {
        entries.extend(
            read_dir(&path)
                .with_context(|| format!("Couldn't list files in {path:?}"))?,
        )
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(entries
        .into_iter()
        .enumerate()
        .map(|(index, file)| InputRow {
            index: index + 1,
            name: file.name,
            is_dir: file.is_dir,
        })
        .collect())
}

/// Generates the text content for the temporary file
/// Since the text will contain file paths(which may not be valid UTF-8)
/// [`RawOsString`] is used instead of a normal UTF-8 [`String`]
fn get_input(files: &[InputRow]) -> RawOsString {
    let list = files
        .iter()
        .map(|res| {
            let mut row = OsString::from(format!("{: >5} ", res.index));
            row.push(res.name.clone().into_os_string());
            if res.is_dir {
                row.push("/")
            }
            row
        })
        .collect::<Vec<OsString>>();

    RawOsString::new(list.join(&OsString::from("\n")))
}

/// Writes the content to a new temporary file and returns a handle
/// Uses [`NamedTempFile`] since we need the to pass the path to the editor
/// This should be fine as the file should have a short lifespan
/// The file will be automatically removed when dropped
fn write_file(file_input: &RawOsString) -> Result<NamedTempFile> {
    let mut file =
        NamedTempFile::new().context("Failed to create temporary file")?;
    file.write_all(file_input.as_raw_bytes())
        .context("Failed to write to temporary file")?;
    Ok(file)
}

fn read_file(path: &Path) -> Result<RawOsString> {
    Ok(RawOsString::assert_from_raw_vec(
        fs::read(path).context("Failed to read temporary file")?,
    ))
}

fn get_editor() -> String {
    env::var(EDITOR_ENV).unwrap_or_else(|_| DEFAULT_EDITOR.into())
}

fn open_editor(editor: &String, file_path: &Path) -> Result<()> {
    Command::new(editor)
        .arg(file_path)
        .status()
        .with_context(|| format!("Failed to open editor {editor:?}"))
        .and_then(|status| {
            if status.success() {
                return Ok(());
            }

            if let Some(code) = status.code() {
                bail!("Editor {editor:?} returned error code {code}")
            } else {
                bail!("Editor {editor:?} returned an error")
            }
        })
}

fn parse_files(input: RawOsString) -> Result<Vec<OutputRow>> {
    input
        .trim_matches(' ')
        .split('\n')
        .filter(|row| row.ne(&""))
        .enumerate()
        .map(|(i, row)| {
            let (index_str, name_str) =
                row.trim_matches(' ')
                    .split_once(' ')
                    .ok_or_else(|| anyhow!("Couldn't find index at row {i}"))?;
            let index_str = index_str.to_str_lossy();
            let index = index_str.parse::<usize>().map_err(|_| {
                anyhow!("Invalid index {index_str:?} at row {i}",)
            })?;
            let name = PathBuf::from(
                name_str.trim_matches(" ").to_owned().into_os_string(),
            );
            Ok(OutputRow { index, name })
        })
        .collect()
}

fn rm_file(file: &InputRow, verbose: bool) -> Result<()> {
    if file.is_dir {
        fs::remove_dir_all(&file.name)
    } else {
        fs::remove_file(&file.name)
    }
    .with_context(|| {
        format!(
            "Error deleting {} {:?}",
            if file.is_dir { "directory" } else { "file" },
            file.name
        )
    })
    .map(|_| {
        if verbose {
            println!(
                "Removed {} {:?}",
                if file.is_dir { "directory" } else { "file" },
                file.name
            )
        }
    })
}
fn mv_file(from: &InputRow, to: &OutputRow, verbose: bool) -> Result<()> {
    fs::rename(&from.name, &to.name)
        .with_context(|| {
            format!(
                "Error moving {} {:?} to {:?}",
                if from.is_dir { "directory" } else { "file" },
                from.name,
                to.name
            )
        })
        .map(|_| {
            if verbose {
                println!(
                    "Moved {} {:?} to {:?}",
                    if from.is_dir { "directory" } else { "file" },
                    from.name,
                    to.name
                )
            }
        })
}

fn update_files(
    input: &[InputRow],
    output: &[OutputRow],
    verbose: bool,
) -> Result<()> {
    let input_idxs: HashSet<_> = input.iter().map(|row| row.index).collect();
    output.iter().enumerate().try_for_each(|(i, output_row)| {
        if !input_idxs.contains(&output_row.index) {
            bail!("Unknown index {} at row {i}", output_row.index)
        } else {
            Ok(())
        }
    })?;

    let output_hash = output
        .iter()
        .map(|row| (row.index, row))
        .collect::<HashMap<_, _>>();

    input.iter().try_for_each(|input_row| -> Result<()> {
        match output_hash.get(&input_row.index) {
            None => rm_file(input_row, verbose),
            Some(output_row) if output_row.name != input_row.name => {
                mv_file(input_row, output_row, verbose)
            }
            _ => Ok(()), // No change
        }
    })
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path_args = get_path_args(args.path);
    let editor = get_editor();

    let input_files = list_files(path_args)?;
    let file_input = get_input(&input_files);

    let file = write_file(&file_input)?;
    let file_path = file.path();
    open_editor(&editor, file_path)?;

    let file_output = read_file(file_path)?;

    let output_files = parse_files(file_output)?;
    update_files(&input_files, &output_files, args.verbose)?;

    Ok(())
}
