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

#[derive(Debug)]
pub enum COutput {
    Err(String),
    Out(String),
    None
}

pub(crate) fn run(c: &str) -> COutput {
    let windows = cfg!(target_os = "windows");

    let output = Command::new(if windows { "cmd" } else { "sh" })
        .arg(if windows { "/C" } else { "-c" })
        .arg(c)
        .output()
        .expect("Command Failed");

    match output.status.code().unwrap() {
        0 => { COutput::Out( String::from_utf8(output.stdout).unwrap() ) },
        _ => { COutput::Err( String::from_utf8(output.stderr).unwrap() ) }
    }
}