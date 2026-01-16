# Project Structure

```
/
├── bios_analyzer_rs/          # Rust BIOS analysis tool
│   ├── src/
│   │   ├── main.rs            # Entry point, orchestrates all analysis
│   │   ├── structures.rs      # Data structures (BiosReport, SPD, etc.)
│   │   ├── patterns.rs        # Binary search patterns
│   │   ├── analysis.rs        # Core analysis (UEFI, SPD, frequencies)
│   │   ├── deep_analysis.rs   # GPU, voltages, timings
│   │   ├── advanced_analysis.rs # Hidden features, SMU commands
│   │   ├── ultra_deep.rs      # H2O unlock, UMC, fan curves
│   │   ├── extreme_analysis.rs # CBS/PBS, STAPM, clock domains
│   │   ├── dpm_analysis.rs    # Dynamic Power Management tables
│   │   ├── hidden_menu.rs     # Hidden BIOS menu discovery
│   │   └── ifr_parser.rs      # IFR (Internal Form Representation) parsing
│   ├── Cargo.toml
│   └── target/                # Build output
│
├── research/                  # Python/C++ research tools
│   ├── patcher.py             # Main BIOS patching script
│   ├── bios_analyzer.py       # Python analysis (quick scans)
│   ├── bios_deep_scan.py      # Extended Python analysis
│   ├── patcher_check.cpp      # C++ verification
│   └── *.fd, *.bin            # BIOS dumps (not in git)
│
├── base/                      # Reference materials
│   ├── memory-clock.txt       # IFR dump (frequency menu options)
│   ├── mgcp.txt               # Samsung SPD specification
│   ├── chicago_registers.h    # Analogix display controller
│   ├── MrcLpddr5.h            # Memory reference code header
│   ├── JESD*.pdf              # JEDEC DDR5/LPDDR5 specs
│   └── *.md, *.txt            # Community research (Russian)
│
├── BIOS_SECRETS.md            # Main reverse engineering documentation
├── README.md                  # Quick start guide
└── *.txt, *.json              # Analysis output files
```

## Module Responsibilities

### Rust Analyzer Modules

| Module | Purpose |
|--------|---------|
| `analysis.rs` | UEFI volumes, SPD structures, frequency tables, power limits |
| `deep_analysis.rs` | GPU P-states, voltage tables, memory timings |
| `advanced_analysis.rs` | SMU commands, hidden feature detection |
| `ultra_deep.rs` | H2O unlock locations, UMC registers, thermal/fan |
| `extreme_analysis.rs` | CBS/PBS options, STAPM, FCLK/UCLK ratios |
| `dpm_analysis.rs` | DPM state tables, PowerPlay entries |
| `hidden_menu.rs` | Locked menu option discovery |
| `ifr_parser.rs` | UEFI IFR opcode parsing |

## Key Patterns

- BIOS structures are duplicated (primary + mirror at ~0x800000 offset)
- SPD signature: `23 11 13 0E` (AMD/Valve LPDDR5)
- Frequency table: sequential bytes `0x51-0x5F` with `0x00` separators
- Power limits stored as milliwatts in little-endian (15W = `98 3A 00 00`)
