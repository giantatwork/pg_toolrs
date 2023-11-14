// use std::env;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use indicatif::{ProgressBar, ProgressStyle};

fn dump(
    db_name: String,
    db_user: String,
    dump_file: Option<String>,
    docker_container: Option<String>,
) {
    let mut command = Command::new("pg_dump");

    match docker_container {
        Some(container) => {
            command.arg("docker").arg("exec").arg("-i").arg(container);
        }
        _ => {}
    }

    let dump_path = dump_file.unwrap_or("db.dump".to_string());

    command
        .arg("-U")
        .arg(&db_user)
        .arg("-Fc")
        .arg("--no-acl")
        .arg("--no-owner")
        .arg(&db_name)
        .stdout(Stdio::piped());

    let mut res = command.spawn().expect("Failed to start command");

    let mut stdout = res.stdout.take().expect("Failed to capture stdout");

    let mut buffer = [0; 4096];

    let mut file = std::fs::File::create(dump_path).expect("Failed to create file");
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {total_bytes} {bytes_per_sec}")
            .expect("Template error"),
    );

    loop {
        match stdout.read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                file.write_all(&buffer[..n])
                    .expect("Failed to write to file");
                pb.inc(n as u64);
            }
            Err(e) => {
                eprint!("Command failed with error {:?}", e);
            }
        }
    }
}

fn main() {
    // let args: Vec<String> = env::args().collect();

    // let arg1 = &args[1];
    // println!("First argument: {}", arg1);

    dump("doorsight".to_string(), "adjan".to_string(), None, None);
}
