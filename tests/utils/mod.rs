use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io, process};
use walkdir::WalkDir;

use os_str_bytes::RawOsString;
use tempfile::{tempdir, TempDir};

#[derive(Debug)]
pub struct Output {
    pub status: process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub fn create_test_dir() -> io::Result<TempDir> {
    let test_dir = tempdir()?;
    // For debugging
    println!("Created test dir {:?}", test_dir.path());
    Ok(test_dir)
}

/// Crates test files and directories
/// Files will have their path as value
/// Directory names must end with a slash
pub fn create_test_files(
    dir: impl AsRef<Path>,
    paths: Vec<impl AsRef<OsStr>>,
) -> Result<(), io::Error> {
    paths.iter().try_for_each(|path| {
        let path_buf = dir.as_ref().join(PathBuf::from(path));
        if path_buf.to_str().unwrap().ends_with('/') {
            fs::create_dir_all(path_buf)
        } else {
            let contents = RawOsString::new(OsString::from(&path));
            fs::write(path_buf, contents.as_raw_bytes())
        }
    })
}

/// Asserts that the files the a directory matches the specified structure
/// Will panic on any extra files/directories, mismatched type, or incorrect
/// content
/// The expected paths are specified as pairs of paths and optional content for
/// files
pub fn assert_test_files(
    dir: impl AsRef<Path>,
    paths: Vec<(impl AsRef<OsStr>, Option<&str>)>,
) {
    let mut result = WalkDir::new(dir.as_ref())
        .into_iter()
        .skip(1) // Skip the direcotry itself
        .map(|res| {
            res.map(|entry| {
                let contents = if entry.file_type().is_dir() {
                    None
                } else {
                    Some(fs::read_to_string(entry.path()).unwrap())
                };
                (
                    entry.path().strip_prefix(dir.as_ref()).unwrap().to_owned(),
                    contents,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut expected: Vec<(PathBuf, Option<String>)> = paths
        .into_iter()
        .map(|(path, contents)| {
            (Into::<PathBuf>::into(&path), contents.map(|s| s.to_owned()))
        })
        .collect();

    result.sort();
    expected.sort();

    println!("Result: {result:#?}");
    println!("Expected: {expected:#?}");

    assert!(result.iter().eq(expected.iter()));
}

pub fn get_bin_path() -> PathBuf {
    env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join(format!("../rsdir{}", env::consts::EXE_SUFFIX))
}

pub fn get_tests_path() -> PathBuf {
    env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join("../../../tests")
}

pub fn get_script_path() -> PathBuf {
    get_tests_path().join("./ed.sh")
}

pub fn run_rsdir(
    dir: impl AsRef<Path>,
    ed_script: &str,
    verbose: bool,
) -> Result<Output, Box<dyn Error>> {
    let bin_path = get_bin_path();
    let ed_path = get_script_path();

    let mut cmd = Command::new(bin_path);
    cmd.current_dir(dir);
    cmd.env("ED_SCRIPT", ed_script);
    cmd.env("EDITOR", ed_path);
    if verbose {
        cmd.args(["--verbose"]);
    }

    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?.trim_end().to_owned();
    let stderr = String::from_utf8(output.stderr)?.trim_end().to_owned();

    println!("status: {}", output.status);
    println!("stdout: {stdout}");
    println!("stderr: {stderr}");

    Ok(Output {
        status: output.status,
        stdout,
        stderr,
    })
}
