# 🐦 Silent Sparrow

**A minimal, self-contained Rust crate that publishes a regularly updated, Ed25519-signed JSON status file.**

Silent Sparrow is designed for services that want to give technically inclined users an independent way to verify that the publishing process is still running normally.

The system consists of two components:
- **Main canary process**: Refreshes the signed status file on a fixed schedule
- **Watchdog process**: Monitors the file and updates the message if updates stop

If the timestamp ever falls behind the expected cadence, the watchdog updates the message to indicate a problem. If the signature no longer validates with the published public key, observers can draw their own conclusions.

**No grace period. Either the bird sings on time, or it doesn't.**

## How It Works

```
┌─────────────────────┐
│  Silent Sparrow     │  Updates every 24h (configurable)
│  (Main Process)     │  Writes: "All systems chirping normally."
└──────────┬──────────┘
           │
           ▼ writes sparrow-song.json
           │
           ▼
┌─────────────────────┐
│  Watchdog Process   │  Checks every 5 min (configurable)
│  (Safety Net)       │  If file > threshold, overwrites message:
└─────────────────────┘  "The nest has gone quiet."
```

**Key Design**: The watchdog is essential. If the main process crashes, it can't update the file to say something is wrong. The watchdog detects staleness and updates the message accordingly.

## Default Messages

The crate uses bird-themed status messages by default:

* **Normal:** "All systems chirping normally:)"
* **Overdue:** "The nest has gone quiet:("
  * *Note: This is written by the watchdog process when the file timestamp exceeds the configured threshold.*

## Example Output (`sparrow-song.json`)

### Normal Operation
```json
{
  "timestamp": "2025-12-12T12:00:00Z",
  "message": "All systems chirping normally:)",
  "signature": "4a3b2c1d9e8f7g6h5i4j3k2l...",
  "public_key": "8f3a1b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a"
}
```

### When Updates Have Stopped
```json
{
  "timestamp": "2025-12-11T12:05:00Z",
  "message": "The nest has gone quiet:(",
  "signature": "WATCHDOG_OVERRIDE",
  "public_key": "8f3a1b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a"
}
```

**Note:** When the watchdog updates the message, the signature becomes `WATCHDOG_OVERRIDE` since the original cryptographic signature is no longer valid for the modified message. The stale timestamp and changed signature both indicate a problem.

## Configuration

Create a `sparrow.toml` file to configure the behavior:

```toml
# Required update interval in hours
interval_hours = 24

# Where to write the signed file
output_path = "sparrow-song.json"

# Optional – override the default messages
message_normal = "All systems chirping normally:)"
message_overdue = "The nest has gone quiet:("

# Optional – publish to HTTPS endpoint
publish_url = "https://example.com/api/canary"
publish_token = "your-bearer-token"
```

## Installation

### From crates.io

```bash
cargo install silent-sparrow
```

### From source

```bash
git clone https://github.com/0xRogu/silentsparrow.git
cd silentsparrow
cargo build --release
```

Binaries will be in `target/release/`:
- `sparrow` - Main canary process
- `sparrow-watchdog` - Watchdog monitor

## Usage

### Running Both Processes

**Main canary process:**
```bash
sparrow --config=sparrow.toml
```

**Watchdog process** (in a separate terminal or service):
```bash
# Arguments: <path> <max_age_hours> <check_interval_seconds>
sparrow-watchdog sparrow-song.json 25 300
```

The watchdog checks every 5 minutes (300 seconds) and marks the canary as overdue if it's older than 25 hours.

### As Systemd Services

Create `/etc/systemd/system/silent-sparrow.service`:
```ini
[Unit]
Description=Silent Sparrow Canary
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/sparrow --config=/etc/silent-sparrow/sparrow.toml
Restart=always
User=sparrow
WorkingDirectory=/var/lib/silent-sparrow

[Install]
WantedBy=multi-user.target
```

Create `/etc/systemd/system/sparrow-watchdog.service`:
```ini
[Unit]
Description=Silent Sparrow Watchdog
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/sparrow-watchdog /var/lib/silent-sparrow/sparrow-song.json 25 300
Restart=always
User=sparrow
WorkingDirectory=/var/lib/silent-sparrow

[Install]
WantedBy=multi-user.target
```

Enable and start both services:
```bash
sudo systemctl enable --now silent-sparrow sparrow-watchdog
```

### As a Library

Add `silent-sparrow` to your `Cargo.toml` dependencies:

```toml
[dependencies]
silent-sparrow = "0.1"
tokio = { version = "1", features = ["full"] }
```

Example integration:

```rust
use silent_sparrow::{Canary, Config};
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_or_default("sparrow.toml")?;
    let mut canary = Canary::new(config.clone());
    
    // Initial update
    if let Err(e) = canary.refresh().await {
        eprintln!("Initial refresh failed: {}", e);
    }
    
    // Run periodic updates
    let mut interval = interval(config.interval_duration());
    loop {
        interval.tick().await;
        if let Err(e) = canary.refresh().await {
            eprintln!("Scheduled refresh failed: {}", e);
        }
    }
}
```

**Remember:** You still need to run the watchdog separately to handle the case where your process crashes.

## Cryptographic Signing

Silent Sparrow uses **Ed25519** for signing. On first run, it generates a key pair and stores the private key securely in your system's config directory:

- **Linux:** `~/.config/silent-sparrow/sparrow.key`
- **macOS:** `~/Library/Application Support/org.silent-sparrow.Silent Sparrow/sparrow.key`
- **Windows:** `%APPDATA%\silent-sparrow\Silent Sparrow\config\sparrow.key`

**Keep this key secure!** Anyone with access to it can forge canary updates.

The public key is included in every status file for verification.

## Verifying Signatures

To verify a canary signature independently:

```bash
# Extract values from the JSON
TIMESTAMP=$(jq -r '.timestamp' sparrow-song.json)
MESSAGE=$(jq -r '.message' sparrow-song.json)
PUBLIC_KEY=$(jq -r '.public_key' sparrow-song.json)
SIGNATURE=$(jq -r '.signature' sparrow-song.json)

# Construct the signed data (timestamp + newline + message + newline + public_key)
printf "%s\n%s\n%s" "$TIMESTAMP" "$MESSAGE" "$PUBLIC_KEY" > data.txt

# Verify with openssl or a Ed25519 verification tool
```

**Note:** If the signature is `WATCHDOG_OVERRIDE`, the watchdog has modified the message, and the original signature is no longer valid. This is intentional and indicates the main process has stopped updating.

## Releases

The CI/CD pipeline is automated. Every git tag matching `vX.Y.Z` automatically triggers:

* Cross-platform build & test (Linux, macOS, Windows)
* Publication to **crates.io**
* **GitHub Release** creation with pre-built binaries

## License

This project is licensed under the **Mozilla Public License 2.0 (MPL-2.0)**. See [LICENSE](LICENSE) for details.

---

## ⚠️ Disclaimer

**This is a purely technical transparency tool.**

It provides no legal protection and makes no representations about what can or cannot be compelled under any jurisdiction's law.

**Use at your own discretion.**
