// use std::env;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
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
        bail!("Path {:?} does not exist", dump_file);
    }

    let mut command;

    match docker_container {
        Some(container) => {
            command = Command::new("docker");
            command
                .arg("exec")
                .arg("-i")
                .arg(container)
                .arg("pg_restore");
        }
        _ => command = Command::new("pg_restore"),
    }

    command
        .arg("-d")
        .arg(db_name)
        .arg("-U")
        .arg(db_user)
        .arg("--if-exists")
        .arg("-c")
        .arg("--no-owner")
        .stdin(Stdio::piped());

    let mut res = command.spawn().expect("Failed to start command");
    let mut stdin = res.stdin.take().expect("Failed to get stdin");

    let mut buffer = [0; 4096];

    let file = File::open(dump_file).expect("Could not open file");
    let file_size = file.metadata().unwrap().len();
    let mut file_reader = BufReader::new(file);

    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {percent}% {pos}/{len} {msg}")
            .expect("Template error"),
    );

    loop {
        match file_reader.read(&mut buffer) {
            Ok(0) => {
                pb.finish();
                break;
            }
            Ok(n) => {
                let result = stdin.write_all(&buffer[..n]);
                match result {
                    Ok(()) => pb.inc(n as u64),
                    Err(err) => {
                        bail!("Error: {}", err)
                    }
                }
            }
            Err(err) => {
                bail!("Error: {}", err)
            }
        }
    }

    match res.wait() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                bail!("Error: {:?}", status.code());
            }
        }
        Err(err) => {
            bail!("Error: {}", err);
        }
    }
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
                pb.finish();
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

    match drop_db("testje", "postgres", Some("pgtest")) {
        Ok(()) => {
            println!("Dropped database")
        }
        Err(err) => println!("Failed to drop database {:?}", err),
    }

    match create_db("testje", "postgres", Some("pgtest")) {
        Ok(()) => {
            println!("Created database")
        }
        Err(err) => println!("Failed to create database {:?}", err),
    }

    let f = Path::new("db.dump");

    match restore("testje", "postgres", f, Some("pgtest")) {
        Ok(()) => {
            println!("Restored database")
        }
        Err(err) => println!("Failed to restore database {:?}", err),
    }
}
