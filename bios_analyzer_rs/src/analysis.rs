//! Analysis functions for BIOS structures

use crate::patterns::*;
use crate::structures::*;
use byteorder::{LittleEndian, ReadBytesExt};
use colored::Colorize;
use std::io::{Cursor, Read};

/// Find all occurrences of a pattern in data
pub fn find_pattern(data: &[u8], pattern: &[u8]) -> Vec<usize> {
    let mut results = Vec::new();
    if pattern.is_empty() || data.len() < pattern.len() {
        return results;
    }
    for i in 0..=(data.len() - pattern.len()) {
        if &data[i..i + pattern.len()] == pattern {
            results.push(i);
        }
    }
    results
}

/// Analyze UEFI Firmware Volumes
pub fn analyze_uefi_volumes(data: &[u8], report: &mut BiosReport) {
    println!("\n{}", "Analyzing UEFI volumes...".dimmed());
    
    // Find _FVH signatures
    let fvh_offsets = find_pattern(data, EFI_FV_SIGNATURE);
    
    for offset in fvh_offsets {
        if offset >= 40 {
            let vol_start = offset - 40; // FVH is at offset 0x28 in volume header
            if vol_start + 0x48 <= data.len() {
                let mut cursor = Cursor::new(&data[vol_start..]);
                
                // Read GUID (16 bytes)
                let mut guid = [0u8; 16];
                if cursor.read_exact(&mut guid).is_ok() {
                    let guid_str = format!(
                        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                        guid[3], guid[2], guid[1], guid[0],
                        guid[5], guid[4], guid[7], guid[6],
                        guid[8], guid[9], guid[10], guid[11],
                        guid[12], guid[13], guid[14], guid[15]
                    );

                    // Read volume length
                    cursor.set_position(0x20);
                    if let Ok(vol_len) = cursor.read_u64::<LittleEndian>() {
                        report.uefi_volumes.push(UefiVolume {
                            offset: vol_start as u64,
                            size: vol_len,
                            vol_type: "FV".to_string(),
                            guid: guid_str,
                        });
                    }
                }
            }
        }
    }
    println!("  Found {} UEFI volumes", report.uefi_volumes.len());
}

/// Analyze SPD structures
pub fn analyze_spd_structures(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing SPD structures...".dimmed());
    
    let spd_offsets = find_pattern(data, SPD_SIGNATURE);
    
    for offset in spd_offsets {
        if offset + 32 <= data.len() {
            let spd_data = &data[offset..offset + 32];
            let vendor = format!("{:02x}{:02x}{:02x}{:02x}", 
                spd_data[4], spd_data[5], spd_data[6], spd_data[7]);
            let tck = spd_data[0x0C];
            let locked = tck == 0x0A;
            
            let spd = SpdStructure {
                offset: offset as u64,
                vendor,
                tck,
                locked,
                raw: hex::encode(&spd_data[..16]),
            };
            
            // Add patch candidate for locked SPD
            if locked {
                report.patches.push(PatchCandidate {
                    offset: (offset + 0x0C) as u64,
                    original: vec![0x0A],
                    patched: vec![0x02],
                    description: "SPD tCK unlock".to_string(),
                    effect: "Enable frequencies above 6400 MT/s".to_string(),
                    risk: "low".to_string(),
                });
            }
            
            report.spd_structures.push(spd);
        }
    }
    println!("  Found {} SPD structures ({} locked)", 
        report.spd_structures.len(),
        report.spd_structures.iter().filter(|s| s.locked).count());
}

/// Analyze frequency tables
pub fn analyze_frequency_tables(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing frequency tables...".dimmed());
    
    // Pattern 0x51 sequence
    for offset in find_pattern(data, FREQ_PATTERN_51) {
        if offset + 48 <= data.len() {
            let mut values = Vec::new();
            let mut cursor = Cursor::new(&data[offset..offset + 48]);
            while let Ok(val) = cursor.read_u16::<LittleEndian>() {
                if val == 0xFFFF { break; }
                values.push(val);
            }
            report.frequency_tables.push(FrequencyTable {
                offset: offset as u64,
                values,
                table_type: "Memory Clock (0x51+)".to_string(),
            });
        }
    }

    // Pattern 0x59 sequence - add patch candidate
    for offset in find_pattern(data, FREQ_PATTERN_59) {
        if offset + 32 <= data.len() {
            report.patches.push(PatchCandidate {
                offset: offset as u64,
                original: vec![0x59, 0x00],
                patched: vec![0x5F, 0x00],
                description: "Frequency remap".to_string(),
                effect: "3200MHz selection -> ~7000 MT/s".to_string(),
                risk: "low".to_string(),
            });
        }
    }
    
    println!("  Found {} frequency tables", report.frequency_tables.len());
}

/// Analyze power management structures
pub fn analyze_power_management(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing power management...".dimmed());
    
    let power_values = [
        (4000u32, "4W - Min TDP"),
        (8000, "8W - Low TDP"),
        (12000, "12W - Medium TDP"),
        (15000, "15W - Default TDP"),
        (18000, "18W - High TDP"),
        (25000, "25W - Boost TDP"),
        (30000, "30W - Max TDP"),
    ];
    
    for (mw, desc) in power_values {
        let pattern = mw.to_le_bytes();
        for offset in find_pattern(data, &pattern) {
            // Verify it's likely a power value (check context)
            if offset >= 4 && offset + 8 <= data.len() {
                let mut cursor = Cursor::new(&data[offset..offset + 4]);
                if let Ok(val) = cursor.read_u32::<LittleEndian>() {
                    if val == mw {
                        report.power_structures.push(PowerStructure {
                            offset: offset as u64,
                            milliwatts: mw,
                            watts: mw / 1000,
                            description: desc.to_string(),
                        });
                        
                        // Add patch candidate for 15W -> 25W
                        if mw == 15000 {
                            report.patches.push(PatchCandidate {
                                offset: offset as u64,
                                original: 15000u32.to_le_bytes().to_vec(),
                                patched: 25000u32.to_le_bytes().to_vec(),
                                description: "Power limit increase".to_string(),
                                effect: "15W -> 25W TDP".to_string(),
                                risk: "high".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    println!("  Found {} power structures", report.power_structures.len());
}

/// Analyze SMU firmware
pub fn analyze_smu(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing SMU firmware...".dimmed());
    
    for offset in find_pattern(data, SMU_MSG_PATTERN) {
        if offset + 64 <= data.len() {
            let end = data[offset..].iter()
                .position(|&b| b == 0)
                .unwrap_or(64)
                .min(64);
            let msg = String::from_utf8_lossy(&data[offset..offset + end]);
            report.smu_info.push(SmuInfo {
                offset: offset as u64,
                description: msg.to_string(),
                msg_id: None,
            });
        }
    }
    
    for offset in find_pattern(data, SMU_FW_PATTERN) {
        if offset + 64 <= data.len() {
            let end = data[offset..].iter()
                .position(|&b| b == 0)
                .unwrap_or(64)
                .min(64);
            let msg = String::from_utf8_lossy(&data[offset..offset + end]);
            report.smu_info.push(SmuInfo {
                offset: offset as u64,
                description: msg.to_string(),
                msg_id: None,
            });
        }
    }
    
    println!("  Found {} SMU references", report.smu_info.len());
}


/// Analyze strings in BIOS
pub fn analyze_strings(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing strings...".dimmed());
    
    let keywords = [
        "Memory", "Clock", "Frequency", "Power", "Voltage", "TDP",
        "CPU", "GPU", "APU", "SMU", "PSP", "Fan", "Thermal", "Boost",
        "STAPM", "PPT", "VDDQ", "VDD2", "Jupiter", "Valve", "Steam",
        "AGESA", "UMC", "FCLK", "MCLK", "Overclock", "Performance",
    ];
    
    // Search ASCII strings
    for keyword in keywords {
        let pattern = keyword.as_bytes();
        let offsets: Vec<u64> = find_pattern(data, pattern)
            .into_iter()
            .map(|o| o as u64)
            .collect();
        if !offsets.is_empty() {
            report.strings.insert(keyword.to_string(), offsets);
        }
    }
    
    // Search UTF-16LE strings
    for keyword in keywords {
        let utf16: Vec<u8> = keyword.encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        let offsets: Vec<u64> = find_pattern(data, &utf16)
            .into_iter()
            .map(|o| o as u64)
            .collect();
        if !offsets.is_empty() {
            let key = format!("{} (UTF16)", keyword);
            report.strings.insert(key, offsets);
        }
    }
    
    println!("  Found {} keyword categories", report.strings.len());
}

/// Analyze GUIDs
pub fn analyze_guids(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing GUIDs...".dimmed());
    
    for known in KNOWN_GUIDS {
        for offset in find_pattern(data, &known.bytes) {
            report.guids.push(GuidInfo {
                offset: offset as u64,
                guid: format!("{:02X?}", known.bytes),
                name: Some(known.name.to_string()),
            });
        }
    }
    
    println!("  Found {} known GUIDs", report.guids.len());
}

/// Analyze numeric tables (potential GPU clocks, voltages)
pub fn analyze_numeric_tables(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing numeric tables...".dimmed());
    
    // Look for GPU frequency patterns (200-1800 MHz range)
    let mut i = 0;
    while i < data.len() - 32 {
        let mut cursor = Cursor::new(&data[i..i + 32]);
        let mut vals = Vec::new();
        let mut valid = true;
        
        for _ in 0..8 {
            if let Ok(v) = cursor.read_u32::<LittleEndian>() {
                if v >= 200 && v <= 1800 && v % 50 == 0 {
                    vals.push(v);
                } else {
                    valid = false;
                    break;
                }
            }
        }
        
        if valid && vals.len() == 8 {
            let unique: std::collections::HashSet<_> = vals.iter().collect();
            if unique.len() >= 4 {
                report.numeric_tables.push(NumericTable {
                    offset: i as u64,
                    values: vals,
                    table_type: "Potential GPU Freq".to_string(),
                });
            }
        }
        i += 4;
    }
    
    println!("  Found {} numeric tables", report.numeric_tables.len());
}

/// Analyze AMD PSP structures
pub fn analyze_amd_psp(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing AMD PSP...".dimmed());
    
    for offset in find_pattern(data, PSP_SIGNATURE) {
        if offset + 16 <= data.len() {
            let mut cursor = Cursor::new(&data[offset + 4..offset + 8]);
            let size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
            report.psp_entries.push(PspEntry {
                offset: offset as u64,
                entry_type: "PSP Directory".to_string(),
                size,
            });
        }
    }
    
    println!("  Found {} PSP entries", report.psp_entries.len());
}

/// Analyze EC firmware
pub fn analyze_ec(data: &[u8], report: &mut BiosReport) {
    println!("{}", "Analyzing EC firmware...".dimmed());
    
    for offset in find_pattern(data, EC_ITE_PATTERN) {
        if offset + 32 <= data.len() {
            let end = data[offset..].iter()
                .position(|&b| b == 0)
                .unwrap_or(32)
                .min(32);
            let desc = String::from_utf8_lossy(&data[offset..offset + end]);
            report.ec_info.push(EcInfo {
                offset: offset as u64,
                description: desc.to_string(),
            });
        }
    }
    
    // Look for "Jupiter" (Steam Deck codename)
    for offset in find_pattern(data, b"Jupiter") {
        report.ec_info.push(EcInfo {
            offset: offset as u64,
            description: "Jupiter (Steam Deck)".to_string(),
        });
    }
    
    println!("  Found {} EC references", report.ec_info.len());
}
