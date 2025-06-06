# hetzner-backup-pull

📦 One-shot backup CLI tool for Hetzner Cloud servers  
Creates a snapshot → exports it → downloads it → optionally compresses, encrypts, and rsyncs it to local or remote destinations.

---

## 🚀 Features

- 📸 Create snapshot of any Hetzner Cloud server
- ⏳ Polls and waits for completion 
- 📤 Obtains signed image export URL
- ⬇️ Streams snapshot tarball directly to disk
- 🗜️ Optional: compress with `xz`
- 🔐 Optional: encrypt with `openssl enc -aes-256-cbc`
- 🚚 Optional: `rsync` to remote NAS/backup server
- 🧾 SHA-256 hash written alongside every archive
- 💡 Designed for automation and scripting

---

## 🧰 Requirements

- Rust (latest stable)
- `openssl` CLI (optional for encryption)
- `rsync` CLI (optional for transfer)
- Hetzner Cloud API token


## 🔧 Installation

```bash
git clone https://github.com/yourname/hetzner-backup-pull
cd hetzner-backup-pull
cargo build --release
```


## 🔐 Configuration: .env 

Create a `.env` file  to store your Hetzner Cloud API token:

```env
HCLOUD_TOKEN=your_hetzner_cloud_api_token_here
```
This variable is automatically loaded at runtime. You may also export it directly in your shell if preferred:

```bash
export HCLOUD_TOKEN=your_hetzner_cloud_api_token_here
```
