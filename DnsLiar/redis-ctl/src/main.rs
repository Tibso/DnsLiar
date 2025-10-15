#![forbid(unsafe_code)]

mod commands;
mod modules;

use crate::{commands::Args, modules::rules};
use dnsliar::config::Config;

use clap::Parser;
use redis::Client;
use std::{fs, process::ExitCode};
use serde_norway::from_str;

fn main() -> ExitCode {
    // Arguments are parsed and stored
    let args = Args::parse();
    let path_to_confile = &args.path_to_confile;

    // First argument should be the 'path_to_confile'
    let redis_addr = {
        let data = match fs::read_to_string(path_to_confile) {
            Ok(data) => data,
            Err(e) => {
                println!("Error reading file from {path_to_confile:?}: {e}");
                return ExitCode::from(78) // CONFIG
            }
        };
        let config: Config = match from_str(data.as_str()) {
            Ok(config) => config,
            Err(e) => {
                println!("Error deserializing config file data: {e}");
                return ExitCode::from(78) // CONFIG
            }
        };
        config.redis_addr.to_string()
    };

    let client = match Client::open(format!("redis://{redis_addr}/")) {
        Ok(client) => client,
        Err(e) => {
            println!("Error probing the Redis server: {e}");
            return ExitCode::from(68) // NOHOST
        }
    };
    let mut con = match client.get_connection() {
        Ok(con) => con,
        Err(e) => {
            println!("Error creating the connection: {e}");
            return ExitCode::from(69) // UNAVAILABLE
        }
    };

    match commands::handle_args(&mut con, args) {
        Err(e) => {
            println!("{e}");
            ExitCode::from(1)
        },
        Ok(exitcode) => exitcode
    }
}
