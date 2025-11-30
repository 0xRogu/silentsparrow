# 🐦 Silent Sparrow

**A minimal, self-contained Rust crate that publishes a regularly updated, Ed25519-signed JSON status file.**

Silent Sparrow is designed for services that want to give technically inclined users an independent way to verify that the publishing process is still running normally.

The file is refreshed on a fixed schedule. If the timestamp ever falls behind the expected cadence, or if the signature no longer validates with the published public key, observers can draw their own conclusions.

**No grace period. Either the bird sings on time, or it doesn’t.**

## Default Messages

The crate uses bird-themed status messages by default:

* **Normal:** "All systems chirping normally."
* **Overdue:** "The nest has gone quiet."
  * *Note: This is automatically used when the last successful update is older than the configured interval.*

## Example Output (`sparrow-song.json`)

### Normal Operation
```json
{
  "timestamp": "2025-11-29T12:00:00Z",
  "message": "All systems chirping normally.",
  "signature": "4a3b2c1d9e8f7g6h5i4j3k2l...",
  "public_key": "8f3a1b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a"
}
```

### When the process has been unable to update on schedule
```json
{
  "timestamp": "2025-11-30T12:05:00Z",
  "message": "The nest has gone quiet.",
  "signature": "e6f7d8c9b0a1f2e3d4c5b6a7...",
  "public_key": "8f3a1b2c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a"
}
```

## Configuration

Create a `sparrow.toml` file to configure the behavior:

```toml
# Required update interval in hours
interval_hours = 24

# Where to write the signed file
output_path = "public/sparrow-song.json"

# Optional – override the default messages
message_normal = "All systems chirping normally."
message_overdue = "The nest has gone quiet."
```

## Usage

### As a Library

Add `silent_sparrow` to your `Cargo.toml` dependencies, then run it within your application:

```rust
use silent_sparrow::{Canary, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_or_default("sparrow.toml")?;
    
    // Run the canary process indefinitely
    Canary::new(config).run_forever().await?;
    
    Ok(())
}
```

### As a Standalone Binary

You can install and run the binary directly:

```bash
cargo install silent-sparrow
silent-sparrow --config sparrow.toml
```

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

It provides no legal protection and makes no representations about what can or cannot be compelled under any jurisdiction’s law.

**Use at your own discretion.**
