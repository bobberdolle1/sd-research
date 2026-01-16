# Tech Stack

## Languages

- **Rust** - Primary analysis tool (`bios_analyzer_rs/`)
- **Python 3** - Patching scripts and quick analysis (`research/`)
- **C++** - Verification tools (`research/patcher_check.cpp`)
- **Shell** - Utility scripts (`research/biosmaker_check.sh`)

## Rust Dependencies (bios_analyzer_rs)

```toml
memmap2 = "0.9"      # Memory-mapped file I/O
regex = "1"          # Pattern matching
serde = "1"          # Serialization
serde_json = "1"     # JSON output
colored = "2"        # Terminal colors
rayon = "1.10"       # Parallel processing
hex = "0.4"          # Hex encoding
byteorder = "1.5"    # Binary parsing
```

## Build Commands

### Rust Analyzer

```bash
cd bios_analyzer_rs
cargo build --release
./target/release/bios_analyzer <bios_file.fd>
```

### Python Patcher

```bash
# On Steam Deck (requires root for flashing)
sudo python3 research/patcher.py [bios_file.fd]
```

## External Tools (Steam Deck)

- `h2offt` - Insyde BIOS flash tool (`/usr/share/jupiter_bios_updater/h2offt`)
- `RUNTIME-PATCHER.efi` - SREP runtime patcher (Boot From File)

## Output Files

- `bios_analysis_report.json` - Structured analysis results
- `extreme_analysis.txt` - Detailed text output
- `bios_patched.fd` / `bios_modded.bin` - Patched BIOS files
