use std::process::Command;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub(crate) fn item_list() -> String {
    String::from_utf8(run_command(r#"gh project item-list 1 --owner rhaskia --format json"#).stdout).unwrap()
}

pub(crate) fn field_list() -> String {
    String::from_utf8(run_command(r#"gh project field-list 1 --owner rhaskia --format json"#).stdout).unwrap()
}

fn run_command(c: &str) -> std::process::Output {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(c)
            .output()
            .expect("Command Failed")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(c)
            .output()
            .expect("Command Failed")
    }
}