use std::process::Command;
use std::fs::OpenOptions;
use std::io::prelude::*;

// pub(crate) fn item_list() -> String {
//     String::from_utf8(run_command(r#"gh project item-list 1 --owner rhaskia --format json"#)).unwrap()
// }
//
// pub(crate) fn field_list() -> String {
//     String::from_utf8(run_command(r#"gh project field-list 1 --owner rhaskia --format json"#).stdout).unwrap()
// }

fn run_command(c: &str) -> String {
    let windows = cfg!(target_os = "windows");

    String::from_utf8(
    Command::new(if windows { "cmd" } else { "sh" })
        .arg(if windows { "/C" } else { "-c" })
        .arg(c)
        .output()
        .expect("Command Failed")
        .stdout
    ).unwrap()
}