use clap::Parser;
use std::{fs, path::PathBuf, process};

const API_TOKEN_ENV: &str = "HCLOUD_TOKEN";

#[derive(Debug, Parser, Clone)]
#[command(
    name = "hetzner-backup-pull",
    version,
    about = "Create, export, download, compress/encrypt & rsync Hetzner Cloud snapshots",
    author,
    after_help = "Example: \
  HCLOUD_TOKEN=xyz cargo run --release -- \\\n  --server-id 123456 \\\n  --backup-dir /mnt/backups/hcloud \\\n  --compress \\\n  --encrypt mysecretpass \\\n  --rsync-target user@nas:/volume/backups/"
)]
pub struct Cli {
    /// Hetzner Cloud *Server* ID to snapshot.
    #[arg(long)]
    pub server_id: u64,

    /// Local directory where the exported snapshot archive will be stored
    /// *before* optional rsync.
    #[arg(long, value_name = "DIR")]
    pub backup_dir: PathBuf,

    /// Use xz compression while streaming download.
    #[arg(long, default_value_t = false)]
    pub compress: bool,

    /// Passphrase for AES-256-CBC encryption.
    #[arg(long, value_name = "PASSPHRASE")]
    pub encrypt: Option<String>,

    /// Optional rsync destination (e.g. `user@nas:/tank/backups/`).
    ///
    /// When omitted, the file simply remains in `backup_dir`.
    #[arg(long, value_name = "DEST")]
    pub rsync_target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub api_token: String,
    pub cli: Cli,
}

impl Config {
    /// Parse CLI + .env + environment.
    pub fn from_args() -> Self {
        if let Err(err) = dotenvy::dotenv() {
            if err.not_found() {
                eprintln!("  No .env file found");
            } else {
                eprintln!("  Could not load .env: {err}");
            }
        }

        let cli = Cli::parse();

        if !cli.backup_dir.exists() {
            if let Err(e) = fs::create_dir_all(&cli.backup_dir) {
                eprintln!(
                    "Cannot create backup directory '{}': {e}",
                    cli.backup_dir.display()
                );
                process::exit(1);
            }
        }

        let api_token = std::env::var(API_TOKEN_ENV).unwrap_or_else(|_| {
            eprintln!("Environment variable {API_TOKEN_ENV} not set (define in .env or export).");
            process::exit(1);
        });

        Self { api_token, cli }
    }
}
