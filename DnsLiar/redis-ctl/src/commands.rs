use crate::rules;

use std::{path::PathBuf, process::ExitCode};
use clap::{Parser, Subcommand};
use redis::{Connection, RedisError};

/// The structure clap will parse
#[derive(Parser)]
#[command(about = "This is a command-line tool used to edit the blacklist", long_about = None)]
pub struct Args {
    /// Path to dnsliar.conf is required
    #[arg(required = true)]
    pub path_to_confile: PathBuf,

    /// Command to process
    #[command(subcommand)]
    #[arg()]
    pub command: Commands
}

/// The commands that are available
#[derive(Subcommand)]
pub enum Commands {
    /// Add a new custom rule
    Add {
        filter: String,
        item: String,
        src: Option<String>,
        ttl: Option<String>
        // ip1: Option<String>,
        // ip2: Option<String>
    },

    /// Delete a rule
    Remove {
        filter: String,
        item: String
        // ip_ver: Option<u8>
    },

    /// Search rules by pattern
    Search {
        pattern: String,
        filter: Option<String>
    },

    /// Disable rules by pattern
    Disable {
        pattern: String,
        filter: Option<String>
    },

    /// Enable rules by pattern
    Enable {
        pattern: String,
        filter: Option<String>
    },

    /// Feed rules to a filter from a file
    FeedFilter {
        path_to_file: PathBuf,
        filter: String,
        src: Option<String>,
        ttl: Option<String>
    },

    /// Feed rules from downloads
    FeedFromDownloads {
       path_to_file: PathBuf,
       ttl: Option<String>
    }

    // /// Display stats about IP addresses that match a pattern
    // ShowStats {pattern: String},

    // /// Clear stats about IP addresses that match a pattern
    // ClearStats {pattern: String},
}

pub fn handle_args(con: &mut Connection, args: Args) -> Result<ExitCode, RedisError> {
    match args.command {
        Commands::Add { filter, item, src, ttl }
            => rules::add(con, &filter, &item, src.as_deref(), ttl.as_deref()),

        Commands::Remove { filter, item }
            => rules::remove(con, &filter, &item),
        
        Commands::Search { pattern, filter }
            => rules::search(con, &pattern, filter.as_deref()),

        Commands::Disable { pattern, filter }
            => rules::enabled(con, &pattern, filter.as_deref(), false),

        Commands::Enable { pattern, filter }
            => rules::enabled(con, &pattern, filter.as_deref(), true),

        Commands::FeedFilter { path_to_file, filter, src, ttl }
            => rules::feed_filter(con, &path_to_file, &filter, src.as_deref(), ttl.as_deref()),

        Commands::FeedFromDownloads { path_to_file, ttl }
            => rules::feed_from_downloads(con, &path_to_file, ttl.as_deref()),

        //Commands::ClearStats { pattern }
        //    => stats::clear(&mut connection, daemon_id, &pattern),
        //
        //Commands::ShowStats { pattern }
        //    => stats::show(&mut connection, daemon_id, &pattern),
    }
}
