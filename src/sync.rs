//! Streaming download → (xz compression) → (openssl encryption) → disk
//! plus optional rsync push.

use crate::error::HbpError;
use indicatif::ProgressBar;
use reqwest::blocking::Response;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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
    let final_path: PathBuf = dest.to_owned();
    let mut hasher = Sha256::new();

    let mut sink: Box<dyn Write> = if let Some(pass) = &opts.encrypt_pass {
        let mut child = Command::new("openssl")
            .args([
                "enc",
                "-aes-256-cbc",
                "-salt",
                "-pbkdf2",
                "-iter",
                "100000",
                "-pass",
                "stdin",
                "-out",
                final_path.to_str().ok_or_else(|| HbpError::Cli("non-UTF8 path".into()))?,
            ])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| HbpError::Cli(format!("spawn openssl: {e}")))?;

        {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .ok_or_else(|| HbpError::Other("missing openssl stdin".into()))?
                .write_all(pass.as_bytes())?;
        }

        Box::new(
            child
                .stdin
                .take()
                .ok_or_else(|| HbpError::Other("missing openssl streaming stdin".into()))?,
        )
    } else {
        Box::new(File::create(&final_path)?)
    };

    if opts.compress {
        sink = Box::new(xz2::write::XzEncoder::new(sink, 6));
    }

    let mut buf = [0u8; 32 * 1024];
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sink.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        pb.inc(n as u64);
    }
    sink.flush()?;

    if let Some(_) = &opts.encrypt_pass {
        drop(sink);

        let status = Command::new("pgrep")
            .args(["-f", &final_path.to_string_lossy()])
            .status()
            .unwrap_or_default();
        if !status.success() {
            return Err(HbpError::Other("openssl enc failed".into()));
        }
    }

    let digest_hex = hex::encode(hasher.finalize());
    let mut digest_file = final_path.clone();
    digest_file.set_extension("sha256");
    std::fs::write(digest_file, &digest_hex)?;

    Ok(digest_hex)
}

pub fn rsync_to(target: &str, file: &Path) -> Result<(), HbpError> {
    let status = Command::new("rsync")
        .args([
            "--archive",
            "--compress",
            "--partial",
            "--progress",
            file.to_str().ok_or_else(|| HbpError::Cli("non-UTF8 path".into()))?,
            target,
        ])
        .status()?;

    if status.success() {
        eprintln!(" rsync complete");
        Ok(())
    } else {
        Err(HbpError::Other(format!("rsync exited with code {status}")))
    }
}
