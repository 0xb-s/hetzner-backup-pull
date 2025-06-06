//! Streaming download → (xz compression) → (openssl encryption) → disk
//! plus optional rsync push.

use crate::error::HbpError;
use cfg_if::cfg_if;
use indicatif::ProgressBar;
use reqwest::blocking::Response;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Command,
};

/// Options that affect streaming pipeline.
#[derive(Clone)]
pub struct SyncOptions {
    pub compress: bool,
    pub encrypt_pass: Option<String>,
}

pub fn stream_to_disk(
    mut resp: Response,
    dest: &Path,
    pb: &ProgressBar,
    opts: &SyncOptions,
) -> Result<String, HbpError> {
    let file = File::create(dest)?;
    let mut writer: Box<dyn Write> = Box::new(file);

    if opts.compress {
        writer = Box::new(xz2::write::XzEncoder::new(writer, 6));
    }

    cfg_if! {
        if #[cfg(unix)] {
            if let Some(pass) = &opts.encrypt_pass {
                let mut child = Command::new("openssl")
                    .args([
                        "enc", "-aes-256-cbc", "-salt",
                        "-pass", "stdin",
                        "-pbkdf2",
                        "-iter", "100000"
                    ])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| HbpError::Cli(format!("Failed to spawn openssl: {e}")))?;


                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(pass.as_bytes())?;
                }

                writer = Box::new(child.stdout.take().unwrap());
            }
        } else {
            if opts.encrypt_pass.is_some() {
                return Err(HbpError::Cli(
                    "--encrypt requires Unix (openssl CLI)".into(),
                ));
            }
        }
    }

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 32 * 1024];
    loop {
        let len = resp.read(&mut buf)?;
        if len == 0 {
            break;
        }
        writer.write_all(&buf[..len])?;
        hasher.update(&buf[..len]);
        pb.inc(len as u64);
    }
    writer.flush()?;

    let digest = hex::encode(hasher.finalize());
    std::fs::write(dest.with_extension("sha256"), digest.as_bytes())?;
    Ok(digest)
}

pub fn rsync_to(target: &str, file: &Path) -> Result<(), HbpError> {
    let status = Command::new("rsync")
        .args([
            "--archive",
            "--partial",
            "--progress",
            file.to_str()
                .ok_or_else(|| HbpError::Cli("Non-UTF8 path".into()))?,
            target,
        ])
        .status()?;

    if status.success() {
        eprintln!(" rsync complete.");
        Ok(())
    } else {
        Err(HbpError::Other(format!("rsync exited with code {status}",)))
    }
}
