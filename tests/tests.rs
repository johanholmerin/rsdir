// The tests in this file all test the behavior of the built binary on real
// (albeit temporary) files. It should therefore be possible to reuse the tests
// for other implementations. The tests do however rely on some POSIX
// functionality: ed is used to programmatically modify the generated files and
// sh, cat, echo & kill are used to test various behaviors. Some tests also
// rely on not being as root as they test failures when removing or renaming
// files.

#[cfg(target_os = "linux")]
use std::ffi::OsString;
#[cfg(target_os = "linux")]
use std::os::unix::prelude::OsStringExt;
use std::path::PathBuf;
use std::process;
use std::process::Command;

mod utils;

#[test]
fn does_nothing() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["foo/", "foo/bar", "baz"])
        .unwrap();
    let output = utils::run_rsdir(&test_dir, "q", true).unwrap();
    utils::assert_test_files(
        &test_dir,
        vec![
            ("foo/", None),
            ("foo/bar", Some("foo/bar")),
            ("baz", Some("baz")),
        ],
    );
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
fn help_flag() {
    let bin_path = utils::get_bin_path();

    let output = Command::new(bin_path).arg("--help").output().unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_ne!(stdout, "");
    assert_eq!(stderr, "");
    assert!(output.status.success());
}

#[test]
fn version_flag() {
    let bin_path = utils::get_bin_path();

    let output = Command::new(bin_path).arg("--version").output().unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_ne!(stdout, "");
    assert_eq!(stderr, "");
    assert!(output.status.success());
}

#[test]
fn unknown_flag() {
    let bin_path = utils::get_bin_path();

    let output = Command::new(bin_path).arg("--asd").output().unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_eq!(stdout, "");
    assert_ne!(stderr, "");
    assert!(!output.status.success());
}

#[test]
/// Tests the default editor in case the EDITOR environment variables isn't set
/// by setting the PATH to the `tests` directory, which contains a `vi` shell
/// script
fn default_editor() {
    let bin_path = utils::get_bin_path();
    let tests_path = utils::get_tests_path();

    let output = Command::new(bin_path)
        .env("PATH", &tests_path)
        .env_remove("EDITOR")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_eq!(stdout, "fake vi");
    assert_eq!(stderr, "");
    assert!(output.status.success());
}

#[test]
fn moves_dir() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["foo/"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/foo/bar\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("bar/", None)]);
    assert_eq!(output.stdout, "Moved directory \"./foo\" to \"./bar/\"");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
fn move_file_error() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/.\\/baz/\\/non-existent\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("baz", Some("baz"))]);
    assert_eq!(output.stdout, "");
    assert!(output.stderr.starts_with(
        "\
        Error: Error moving file \"./baz\" to \"/non-existent\"

Caused by:"
    ));
    assert!(!output.status.success());
}

#[test]
fn move_dir_error() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz/"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/.\\/baz/\\/non-existent\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("baz/", None)]);
    assert_eq!(output.stdout, "");
    assert!(output.stderr.starts_with(
        "\
        Error: Error moving directory \"./baz\" to \"/non-existent/\"

Caused by:"
    ));
    assert!(!output.status.success());
}

#[test]
fn moves_file() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/baz/boop\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("boop", Some("baz"))]);
    assert_eq!(output.stdout, "Moved file \"./baz\" to \"./boop\"");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
fn non_relative_path() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/\\.\\/baz/boop\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("boop", Some("baz"))]);
    assert_eq!(output.stdout, "Moved file \"./baz\" to \"boop\"");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
/// Checks the content of the generated file using `cat` to print to stdout
fn tmp_file_content() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz", "xox", "lol", "dir/"])
        .unwrap();

    let bin_path = utils::get_bin_path();

    let output = Command::new(bin_path)
        .current_dir(&test_dir)
        .env("EDITOR", "cat")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    utils::assert_test_files(
        &test_dir,
        vec![
            ("baz", Some("baz")),
            ("dir/", None),
            ("xox", Some("xox")),
            ("lol", Some("lol")),
        ],
    );
    assert_eq!(
        stdout,
        "    1 ./baz
    2 ./dir/
    3 ./lol
    4 ./xox"
    );
    assert_eq!(stderr, "");
    assert!(output.status.success());
}

#[test]
fn deletes_dir() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["foo/"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "1d\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, Vec::<(&str, Option<&str>)>::new());
    assert_eq!(output.stdout, "Removed directory \"./foo\"");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
fn deletes_file() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "1d\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, Vec::<(&str, Option<&str>)>::new());
    assert_eq!(output.stdout, "Removed file \"./baz\"");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
fn delete_dir_error() {
    let output = utils::run_rsdir(
        "/",
        "/dev\n\
         d\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    assert_eq!(output.stdout, "");
    assert!(output.stderr.starts_with(
        "\
        Error: Error deleting directory \"./dev\"

Caused by:"
    ));
    assert!(!output.status.success());
}

#[test]
fn delete_file_error() {
    let output = utils::run_rsdir(
        "/dev",
        "/null\n\
         d\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    assert_eq!(output.stdout, "");
    assert!(output.stderr.starts_with(
        "\
        Error: Error deleting file \"./null\"

Caused by:"
    ));
    assert!(!output.status.success());
}

#[test]
fn multiple_args() {
    let test_dir1 = utils::create_test_dir().unwrap();
    let test_dir2 = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir1, vec!["baz", "bop"]).unwrap();
    utils::create_test_files(&test_dir2, vec!["foo", "pop"]).unwrap();

    let bin_path = utils::get_bin_path();
    let ed_path = utils::get_script_path();

    let output = Command::new(bin_path)
        .args([test_dir1.path(), test_dir2.path()])
        .env("EDITOR", "/non-existent")
        .env(
            "ED_SCRIPT",
            "/baz\n\
             d\n\
             /foo\n\
             d\n\
             w\n\
             q",
        )
        .env("EDITOR", ed_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    utils::assert_test_files(&test_dir1, vec![("bop", Some("bop"))]);
    utils::assert_test_files(&test_dir2, vec![("pop", Some("pop"))]);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
    assert!(output.status.success());
}

#[test]
fn unknown_index() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/1/2\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("baz", Some("baz"))]);
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "Error: Unknown index 2 at row 0");
    assert!(!output.status.success());
}

#[test]
fn invalid_index() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/1/x\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("baz", Some("baz"))]);
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "Error: Invalid index \"x\" at row 0");
    assert!(!output.status.success());
}

#[test]
fn editor_failure() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let ed_path = utils::get_script_path();
    let output = utils::run_rsdir(
        &test_dir,
        // Searching for a non-existent string will cause ed to return an error
        "s/missing/never\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("baz", Some("baz"))]);
    assert_eq!(output.stdout, "");
    assert!(output.stderr.starts_with(&format!(
        "Error: Editor {ed_path:?} returned error code "
    )));
    assert!(!output.status.success());
}

#[test]
/// Tests the error handling for when the editor is killed by a signal, in which
/// case no exit code will be returned
/// Uses the `kill.sh` script which kills itself
fn editor_killed() {
    let test_dir = utils::create_test_dir().unwrap();
    let bin_path = utils::get_bin_path();
    let editor_path = utils::get_tests_path().join("./kill.sh");

    let output = Command::new(bin_path)
        .current_dir(&test_dir)
        .env("EDITOR", &editor_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        format!("Error: Editor {editor_path:?} returned an error")
    );
    assert!(!output.status.success());
}

#[test]
fn missing_editor() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();

    let bin_path = utils::get_bin_path();

    let output = Command::new(bin_path)
        .current_dir(&test_dir)
        .env("EDITOR", "/non-existent")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();
    println!("status: {}", output.status);
    println!("stdout: {stdout}");
    println!("stderr: {stderr}");

    utils::assert_test_files(&test_dir, vec![("baz", Some("baz"))]);
    assert_eq!(stdout, "");
    assert!(stderr.starts_with(
        "\
Error: Failed to open editor \"/non-existent\"

Caused by:"
    ));
    assert!(!output.status.success());
}

#[test]
fn missing_path() {
    let bin_path = utils::get_bin_path();
    let ed_path = utils::get_script_path();

    let output = Command::new(bin_path)
        .arg("/non-existent")
        .env("ED_SCRIPT", "q")
        .env("EDITOR", ed_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_eq!(stdout, "");
    assert!(stderr.starts_with(
        "\
Error: Couldn't list files in \"/non-existent\"

Caused by:"
    ));
    assert!(!output.status.success());
}

#[test]
/// Tests the error handling when failing to create a temporary file by setting
/// `TMPDIR` to a non-existent path
fn create_tmpfile_error() {
    let test_dir = utils::create_test_dir().unwrap();
    let bin_path = utils::get_bin_path();
    let ed_path = utils::get_script_path();

    let output = Command::new(bin_path)
        .current_dir(&test_dir)
        .env("TMPDIR", "/non-existent")
        .env("ED_SCRIPT", "q")
        .env("EDITOR", ed_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_owned();
    let stderr = String::from_utf8(output.stderr)
        .unwrap()
        .trim_end()
        .to_owned();

    assert_eq!(stdout, "");
    assert!(stderr.starts_with(
        "\
Error: Failed to create temporary file

Caused by:
    No such file or directory (os error 2) at path \"/non-existent/"
    ));
    assert!(!output.status.success());
}

#[test]
fn non_verbose() {
    let test_dir = utils::create_test_dir().unwrap();
    utils::create_test_files(&test_dir, vec!["baz"]).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "s/baz/boop\n\
         w\n\
         q",
        false,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("boop", Some("baz"))]);
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}

#[test]
fn removes_temp_file() {
    let bin_path = utils::get_bin_path();

    let file_path = String::from_utf8(
        process::Command::new(bin_path)
            .env("EDITOR", "echo")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert!(!PathBuf::from(file_path).exists());
}

#[test]
#[cfg(target_os = "linux")]
/// Tests that paths that are not valid Unicode are handled correctly
/// Does not apply to macOS(HFS+ or APFS) where paths are always valid Unicode
fn byte_path_linux() {
    let file_path_bytes = vec![
        97, 0o017, 0o254, 0o001, 0o103, 0o326, 0o144, 0o203, 0o261, 0o154,
        0o065, 0o053, 0o167,
    ];
    // Ensure that the test path is invalid UTF-8
    String::from_utf8(file_path_bytes.clone()).unwrap_err();
    let file_path_os: OsString = OsStringExt::from_vec(file_path_bytes);

    let test_dir = utils::create_test_dir().unwrap();
    let path_buf = test_dir.path().join(PathBuf::from(file_path_os));
    std::fs::create_dir_all(path_buf).unwrap();
    let output = utils::run_rsdir(
        &test_dir,
        "d\n\
         i\n\
         1 ./baz\n\
         .\n\
         w\n\
         q",
        true,
    )
    .unwrap();
    utils::assert_test_files(&test_dir, vec![("baz", None)]);
    assert_eq!(
        output.stdout,
        "Moved directory \
        \"./a\\u{f}\\xAC\\u{1}C\\xD6d\\x83\\xB1l5+w\" to \"./baz\""
    );
    assert_eq!(output.stderr, "");
    assert!(output.status.success());
}
