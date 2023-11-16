// TODO aparte modules maken voor functies en helpers etc.
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Error};
use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

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
    dump_file: &str,
    docker_container: Option<&str>,
) -> Result<(), Error> {
    if !Path::new(dump_file).exists() {
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
    // https://blog.ediri.io/lets-build-a-cli-in-rust

    #[derive(Parser)]
    #[command(author, version)]
    #[command(about = "Tool to perform various db actions", long_about = "blah")]

    struct Cli {
        #[command(subcommand)]
        command: Option<Commands>,
    }

    #[derive(Subcommand)]
    enum Commands {
        Dump(Dump),
        Restore(Restore),
        Drop(Drop),
        Create(Create),
    }

    #[derive(Args)]
    struct Dump {
        db_name: String,
        db_user: String,
        dump_file: Option<String>,
        docker_container: Option<String>,
    }

    #[derive(Args)]
    struct Restore {
        db_name: String,
        db_user: String,
        dump_file: String,
        docker_container: Option<String>,
    }

    #[derive(Args)]
    struct Drop {
        db_name: String,
        db_user: String,
        docker_container: Option<String>,
    }

    #[derive(Args)]
    struct Create {
        db_name: String,
        db_user: String,
        docker_container: Option<String>,
    }

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Dump(cmd)) => {
            match dump(
                &cmd.db_name,
                &cmd.db_user,
                cmd.dump_file.as_deref(),
                cmd.docker_container.as_deref(),
            ) {
                Ok(()) => {
                    println!("Dumped database {}", cmd.db_name);
                }
                Err(err) => {
                    println!("Failed to dump database {}, error: {}", cmd.db_name, err);
                }
            }
        }
        Some(Commands::Restore(cmd)) => {
            match restore(
                &cmd.db_name,
                &cmd.db_user,
                &cmd.dump_file,
                cmd.docker_container.as_deref(),
            ) {
                Ok(()) => {
                    println!("Restored database {}", cmd.db_name);
                }
                Err(err) => {
                    println!("Failed to restore database {}, error: {}", cmd.db_name, err);
                }
            }
        }
        Some(Commands::Drop(cmd)) => {
            match drop_db(
                &cmd.db_name,
                &cmd.db_user,
                cmd.docker_container.as_deref(),
            ){
                Ok(()) => {
                    println!("Dropped database {}", cmd.db_name);
                }
                Err(err) => {
                    println!("Failed to drop database {}, error: {}", cmd.db_name, err);
                }
            }
        }
        Some(Commands::Create(cmd)) => {
            match create_db(
                &cmd.db_name,
                &cmd.db_user,
                cmd.docker_container.as_deref(),
            ){
                Ok(()) => {
                    println!("Created database {}", cmd.db_name);
                }
                Err(err) => {
                    println!("Failed to create database {}, error: {}", cmd.db_name, err);
                }
            }
        }
        None => {}
    }

}
