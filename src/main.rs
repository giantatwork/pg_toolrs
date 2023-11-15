// use std::env;
use std::{
    io::{Read, Write},
    process::{Command, ExitStatus, Stdio},
};

use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
enum DropDBError {
    CommandFailed(ExitStatus),
    IoError(std::io::Error),
}

fn drop_db(
    db_name: &str,
    db_user: &str,
    docker_container: Option<&str>,
) -> Result<(), DropDBError> {
    let mut command = Command::new("dropdb");

    match docker_container {
        Some(container) => {
            command.arg("docker").arg("exec").arg("-i").arg(container);
        }
        _ => {}
    }
    command.arg(db_name).arg("-U").arg(db_user);

    let result = command.output();
    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(DropDBError::CommandFailed(output.status))
            }
        }
        Err(err) => Err(DropDBError::IoError(err)),
    }
}

fn dump(db_name: &str, db_user: &str, dump_file: Option<&str>, docker_container: Option<&str>) {
    let mut command = Command::new("pg_dump");

    match docker_container {
        Some(container) => {
            command.arg("docker").arg("exec").arg("-i").arg(container);
        }
        _ => {}
    }

    let dump_path = dump_file.unwrap_or("db.dump");

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

    dump("doorsight", "adjan", None, None);
}
