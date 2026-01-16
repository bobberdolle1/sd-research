//! Data structures for BIOS analysis

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BiosReport {
    pub filename: String,
    pub size: usize,
    pub uefi_volumes: Vec<UefiVolume>,
    pub spd_structures: Vec<SpdStructure>,
    pub frequency_tables: Vec<FrequencyTable>,
    pub power_structures: Vec<PowerStructure>,
    pub smu_info: Vec<SmuInfo>,
    pub strings: BTreeMap<String, Vec<u64>>,
    pub guids: Vec<GuidInfo>,
    pub numeric_tables: Vec<NumericTable>,
    pub psp_entries: Vec<PspEntry>,
    pub ec_info: Vec<EcInfo>,
    pub patches: Vec<PatchCandidate>,
}

impl BiosReport {
    pub fn new(filename: &str, size: usize) -> Self {
        Self {
            filename: filename.to_string(),
            size,
            uefi_volumes: Vec::new(),
            spd_structures: Vec::new(),
            frequency_tables: Vec::new(),
            power_structures: Vec::new(),
            smu_info: Vec::new(),
            strings: BTreeMap::new(),
            guids: Vec::new(),
            numeric_tables: Vec::new(),
            psp_entries: Vec::new(),
            ec_info: Vec::new(),
            patches: Vec::new(),
        }
    }

    pub fn print(&self) {
        use colored::*;
        
        // UEFI Volumes
        println!("\n{}", "═".repeat(80).cyan());
        println!("{}", " 1. UEFI VOLUMES".bold().yellow());
        println!("{}", "═".repeat(80).cyan());
        for vol in &self.uefi_volumes {
            println!("  {} @ 0x{:08X} - Size: 0x{:X}, Type: {}", 
                "Volume".green(), vol.offset, vol.size, vol.vol_type);
        }
        
        // SPD
        println!("\n{}", "═".repeat(80).cyan());
        println!("{}", " 2. SPD STRUCTURES (Memory)".bold().yellow());
        println!("{}", "═".repeat(80).cyan());
        for spd in &self.spd_structures {
            let status = if spd.locked { "LOCKED".red() } else { "UNLOCKED".green() };
            println!("  @ 0x{:08X}: tCK=0x{:02X} [{}] vendor={}", 
                spd.offset, spd.tck, status, spd.vendor);
        }
        
        // Frequency Tables
        println!("\n{}", "═".repeat(80).cyan());
        println!("{}", " 3. FREQUENCY TABLES".bold().yellow());
        println!("{}", "═".repeat(80).cyan());
        for ft in &self.frequency_tables {
            println!("  @ 0x{:08X}: {:?}", ft.offset, ft.values);
        }
        
        // Power
        println!("\n{}", "═".repeat(80).cyan());
        println!("{}", " 4. POWER MANAGEMENT".bold().yellow());
        println!("{}", "═".repeat(80).cyan());
        for pw in &self.power_structures {
            println!("  @ 0x{:08X}: {}W ({}mW) - {}", 
                pw.offset, pw.watts, pw.milliwatts, pw.description);
        }
        
        // SMU
        println!("\n{}", "═".repeat(80).cyan());
        println!("{}", " 5. SMU FIRMWARE".bold().yellow());
        println!("{}", "═".repeat(80).cyan());
        for smu in &self.smu_info {
            println!("  @ 0x{:08X}: {}", smu.offset, smu.description);
        }
        
        // Patches
        println!("\n{}", "═".repeat(80).cyan());
        println!("{}", " PATCH CANDIDATES".bold().green());
        println!("{}", "═".repeat(80).cyan());
        for patch in &self.patches {
            let risk = match patch.risk.as_str() {
                "low" => "LOW".green(),
                "medium" => "MEDIUM".yellow(),
                "high" => "HIGH".red(),
                _ => patch.risk.normal(),
            };
            println!("  [{}] @ 0x{:08X}: {} -> {}", 
                risk, patch.offset, patch.description, patch.effect);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UefiVolume {
    pub offset: u64,
    pub size: u64,
    pub vol_type: String,
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpdStructure {
    pub offset: u64,
    pub vendor: String,
    pub tck: u8,
    pub locked: bool,
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrequencyTable {
    pub offset: u64,
    pub values: Vec<u16>,
    pub table_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerStructure {
    pub offset: u64,
    pub milliwatts: u32,
    pub watts: u32,
    pub description: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SmuInfo {
    pub offset: u64,
    pub description: String,
    pub msg_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuidInfo {
    pub offset: u64,
    pub guid: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumericTable {
    pub offset: u64,
    pub values: Vec<u32>,
    pub table_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PspEntry {
    pub offset: u64,
    pub entry_type: String,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EcInfo {
    pub offset: u64,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchCandidate {
    pub offset: u64,
    pub original: Vec<u8>,
    pub patched: Vec<u8>,
    pub description: String,
    pub effect: String,
    pub risk: String,
}
