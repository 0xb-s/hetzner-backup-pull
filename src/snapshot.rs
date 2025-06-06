//! High-level orchestration of snapshot → export → download.

use crate::{
    api::{build_client, create_snapshot, export_image, stream_download, wait_for_action},
    config::Config,
    error::HbpError,
    sync::{SyncOptions, stream_to_disk},
};
use chrono::{SecondsFormat, Utc};
use indicatif::{ProgressBar, ProgressStyle};

pub fn run(cfg: &Config) -> Result<(), HbpError> {
    let client = build_client(&cfg.api_token)?;

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let description = format!("hbp-{}", timestamp);

    eprintln!(" Creating snapshot of server {} …", cfg.cli.server_id);
    let (image_id, action) = create_snapshot(&client, cfg.cli.server_id, &description)?;
    wait_for_action(&client, &action)?;
    eprintln!(" Snapshot {image_id} ready.");

    eprintln!("  Requesting export link …");
    let (download_url, export_action) = export_image(&client, image_id)?;
    wait_for_action(&client, &export_action)?;
    eprintln!(" Export link received.");

    eprintln!("   Streaming download …");
    let response = stream_download(&client, &download_url)?;

    let filename = format!("snapshot-{image_id}.tar");
    let local_path = cfg.cli.backup_dir.join(&filename);

    let pb = ProgressBar::new(
        response.content_length().unwrap_or(0), // 0 = “unknown” (still shows spinner)
    );
    pb.set_style(
        ProgressStyle::with_template("{spinner} [{elapsed_precise}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let sync_opts = SyncOptions {
        compress: cfg.cli.compress,
        encrypt_pass: cfg.cli.encrypt.clone(),
    };

    stream_to_disk(response, &local_path, &pb, &sync_opts)?;

    pb.finish_with_message("  Download complete");

    if let Some(dest) = &cfg.cli.rsync_target {
        crate::sync::rsync_to(dest, &local_path)?;
    }

    eprintln!(" All done - exit 0");
    Ok(())
}
