// use std::env;
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use indicatif::{ProgressBar, ProgressStyle};

use anyhow::{bail, Error};

fn create_db(db_name: &str, db_user: &str, docker_container: Option<&str>) -> Result<(), Error> {
    let mut command: Command;

    match docker_container {
        Some(container) => {
            command = Command::new("docker");
            command.arg("exec").arg("-i").arg(container).arg("createdb");
        }
        _ => {
            command = Command::new("createdb");
        }
    }

    command.arg(db_name).arg("-U").arg(db_user);

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                bail!("Error: {}", output.status.to_string())
            }
        }
        Err(err) => bail!("Error: {}", err),
    }
}

fn drop_db(db_name: &str, db_user: &str, docker_container: Option<&str>) -> Result<(), Error> {
    let mut command: Command;

    match docker_container {
        Some(container) => {
            command = Command::new("docker");
            command.arg("exec").arg("-i").arg(container).arg("dropdb");
        }
        _ => {
            command = Command::new("dropdb");
        }
    }

    command.arg(db_name).arg("-U").arg(db_user);

    let result = command.output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                bail!("Error: {}", output.status.to_string())
            }
        }
        Err(err) => bail!("Error: {}", err),
    }
}

fn restore(
    db_name: &str,
    db_user: &str,
    dump_file: &Path,
    docker_container: Option<&str>,
) -> Result<(), Error> {
    if !dump_file.exists() {
        bail!("Path does not exist");
    }

    Ok(())
}

fn dump(
    db_name: &str,
    db_user: &str,
    dump_file: Option<&str>,
    docker_container: Option<&str>,
) -> Result<(), Error> {
    let mut command;

    match docker_container {
        Some(container) => {
            command = Command::new("docker");
            command.arg("exec").arg("-i").arg(container).arg("pg_dump");
        }
        _ => {
            command = Command::new("pg_dump");
        }
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
                let result = file.write_all(&buffer[..n]);
                match result {
                    Ok(()) => {
                        pb.inc(n as u64);
                    }
                    Err(err) => bail!("Error: {}", err),
                }
            }
            Err(err) => bail!("Error: {}", err),
        }
    }

    Ok(())
}

fn main() {
    // let args: Vec<String> = env::args().collect();

    // let arg1 = &args[1];
    // println!("First argument: {}", arg1);

    // let _ = dump("doorsight", "adjan", None, None).expect("Failed to dump database");

    match create_db("testje", "postgres", Some("pgtest")) {
        Ok(()) => {
            println!("Created database")
        }
        Err(err) => println!("Failed to create database {:?}", err),
    }

    match drop_db("testje", "postgres", Some("pgtest")) {
        Ok(()) => {
            println!("Dropped database")
        }
        Err(err) => println!("Failed to drop database {:?}", err),
    }
}
