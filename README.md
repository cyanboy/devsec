# DevSec 🛡️
A fast, lightweight **DevSecOps CLI tool** for fetching and analyzing repositories from GitLab.
It helps **automate security checks**, **analyze repository metadata**, and **search repositories efficiently** using SQLite FTS5.

## Installation 🚀
### Install via Cargo
```sh
cargo install devsec
```
### Build from Source
```sh
git clone https://github.com/yourusername/devsec.git
cd devsec
cargo build --release
```

## Usage ⚡️

### Update database with data from GitLab
```sh
devsec gitlab update --auth <GITLAB TOKEN> --group-id <GITLAB GROUP ID>
```

## Configuration ⚙️

DevSec stores its SQLite database in:
- **Linux**: `~/.local/share/devsec/devsec.sqlite`
- **macOS**: `~/Library/Application Support/devsec/devsec.sqlite`
- **Windows**: `%APPDATA%\devsec\devsec.sqlite`

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any Contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
