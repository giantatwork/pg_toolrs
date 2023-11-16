// TODO aparte modules maken voor functies en helpers etc.
use clap::Parser;

use crate::database::create_db;
use crate::database::drop_db;
use crate::database::dump;
use crate::database::restore;

use crate::cli::Cli;
use crate::cli::Commands;

mod cli;
mod database;

fn main() {
    // https://blog.ediri.io/lets-build-a-cli-in-rust

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
            match drop_db(&cmd.db_name, &cmd.db_user, cmd.docker_container.as_deref()) {
                Ok(()) => {
                    println!("Dropped database {}", cmd.db_name);
                }
                Err(err) => {
                    println!("Failed to drop database {}, error: {}", cmd.db_name, err);
                }
            }
        }
        Some(Commands::Create(cmd)) => {
            match create_db(&cmd.db_name, &cmd.db_user, cmd.docker_container.as_deref()) {
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
