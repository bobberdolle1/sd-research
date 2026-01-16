# Product Overview

Steam Deck BIOS reverse engineering and memory overclocking toolkit.

## Purpose

Enable LPDDR5 memory overclocking on Steam Deck LCD from stock 6400 MT/s to ~7000 MT/s by:
- Unlocking SPD (Serial Presence Detect) tCK byte restrictions
- Remapping frequency table values in BIOS
- Documenting hidden CBS/PBS menu options

## Target Hardware

- Steam Deck LCD (Jupiter/Aerith APU)
- Samsung K3LKBKB0BM-MGCP LPDDR5 memory (B-die preferred)
- BIOS versions: F7A0131, F7A0133

## Key Deliverables

1. `patcher.py` - BIOS patching tool for memory unlock
2. `bios_analyzer_rs` - Rust-based deep BIOS analysis tool
3. `BIOS_SECRETS.md` - Comprehensive reverse engineering documentation
4. Reference materials in `base/` (SPD specs, IFR dumps, community research)

## Language Note

Primary documentation is in Russian (target community). Code comments may be mixed Russian/English.
