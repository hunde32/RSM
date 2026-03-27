# RSM (Rusty Symlink Manager)

A fast, safe, and beautifully formatted CLI tool for managing symlinks, written in Rust. Designed with systems engineers and dotfile enthusiasts in mind.

## Features

- **Safety First:** Prevents accidental overwrites of real files.
- **Dry-Run Mode:** Simulate operations before committing changes.
- **Beautiful UI:** Provides clear visual feedback and progress bars.
- **Config-Driven:** Uses a straightforward `rsm.toml` file for mapping.

## Installation

### From Source

```bash
git clone https://github.com/hunde32/rsm.git
cd rsm
cargo build --release
sudo cp target/release/rsm /usr/local/bin/
```
