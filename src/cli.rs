use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
#[command(
    about = "Tool to perform various db actions",
    long_about = "blahdeeblah"
)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Dump(Dump),
    Restore(Restore),
    Drop(Drop),
    Create(Create),
}

#[derive(Args)]
pub struct Dump {
    pub db_name: String,
    pub db_user: String,
    pub dump_file: Option<String>,
    pub docker_container: Option<String>,
}

#[derive(Args)]
pub struct Restore {
    pub db_name: String,
    pub db_user: String,
    pub dump_file: String,
    pub docker_container: Option<String>,
}

#[derive(Args)]
pub struct Drop {
    pub db_name: String,
    pub db_user: String,
    pub docker_container: Option<String>,
}

#[derive(Args)]
pub struct Create {
    pub db_name: String,
    pub db_user: String,
    pub docker_container: Option<String>,
}
