//! Ultra deep analysis - H2O unlock, UMC, Fan curves, Thermal thresholds

use byteorder::{LittleEndian, ReadBytesExt};
use colored::Colorize;
use std::io::Cursor;

pub fn ultra_deep_analysis(data: &[u8]) {
    println!("\n{}", "═".repeat(80).bright_yellow());
    println!("{}", " ULTRA DEEP ANALYSIS".bold().bright_yellow());
    println!("{}", "═".repeat(80).bright_yellow());

    // 1. H2O Unlock mechanism
    analyze_h2o_unlock(data);
    
    // 2. UMC (Unified Memory Controller)
    analyze_umc(data);
    
    // 3. All fan curves
    analyze_all_fan_curves(data);
    
    // 4. Thermal thresholds and throttling
    analyze_thermal_thresholds(data);
    
    // 5. SMU message IDs
    analyze_smu_messages(data);
    
    // 6. Power tables detailed
    analyze_power_tables_detailed(data);
    
    // 7. GPU P-States
    analyze_gpu_pstates(data);
    
    // 8. APCB/APOB structures
    analyze_apcb_apob(data);
}

fn analyze_h2o_unlock(data: &[u8]) {
    println!("\n{}", "  [H2O UNLOCK MECHANISM]".bold().bright_green());
    
    // Search for H2O related strings
    let h2o_patterns = [
        (b"H2OAuthUnlock".as_slice(), "Auth Unlock Function"),
        (b"H2OChannel".as_slice(), "Channel Communication"),
        (b"H2OSetup".as_slice(), "Setup Module"),
        (b"H2OUVE".as_slice(), "UEFI Variable Editor"),
        (b"H2OFFT".as_slice(), "Flash Tool"),
        (b"Insyde".as_slice(), "Insyde BIOS"),
        (b"SetupUtility".as_slice(), "Setup Utility"),
        (b"AdvancedMenu".as_slice(), "Advanced Menu"),
        (b"HiddenMenu".as_slice(), "Hidden Menu"),
        (b"UnlockSetup".as_slice(), "Unlock Setup"),
        (b"AdminPassword".as_slice(), "Admin Password"),
        (b"UserPassword".as_slice(), "User Password"),
    ];
    
    for (pattern, desc) in h2o_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {} [{}]: {} matches", desc.green(), 
                String::from_utf8_lossy(pattern), matches.len());
            for &offset in matches.iter().take(3) {
                let ctx = get_context_string(data, offset, 80);
                println!("      @ 0x{:08X}: {}", offset, ctx.dimmed());
            }
        }
    }
    
    // Look for password/unlock related byte patterns
    println!("\n    {}", "Searching for unlock mechanisms...".yellow());
    
    // Common unlock patterns: comparing against hardcoded values
    // Look for CMP instructions followed by JE/JNE
    let mut unlock_candidates = Vec::new();
    for i in 0..data.len().saturating_sub(16) {
        // Pattern: password check (comparing bytes)
        if data[i] == 0x3D || data[i] == 0x3C { // CMP AL/EAX
            let ctx = &data[i..i.min(data.len()-8)+8];
            // Check if followed by conditional jump
            if ctx.len() >= 6 && (ctx[5] == 0x74 || ctx[5] == 0x75) { // JE/JNE
                unlock_candidates.push(i);
            }
        }
    }
    println!("    Found {} potential unlock check locations", unlock_candidates.len());
}


fn analyze_umc(data: &[u8]) {
    println!("\n{}", "  [UMC - UNIFIED MEMORY CONTROLLER]".bold().bright_green());
    
    // UMC related patterns
    let umc_patterns = [
        (b"UMC".as_slice(), "UMC Reference"),
        (b"MemClk".as_slice(), "Memory Clock"),
        (b"DramTiming".as_slice(), "DRAM Timing"),
        (b"MemPstate".as_slice(), "Memory P-State"),
        (b"UmcCh".as_slice(), "UMC Channel"),
        (b"DctCfg".as_slice(), "DCT Config"),
        (b"MemTraining".as_slice(), "Memory Training"),
        (b"PhyInit".as_slice(), "PHY Init"),
        (b"DqsTraining".as_slice(), "DQS Training"),
        (b"WrLvl".as_slice(), "Write Leveling"),
        (b"RdDqs".as_slice(), "Read DQS"),
        (b"WrDqs".as_slice(), "Write DQS"),
        (b"Vref".as_slice(), "Voltage Reference"),
        (b"ZQ".as_slice(), "ZQ Calibration"),
        (b"MR0".as_slice(), "Mode Register 0"),
        (b"MR1".as_slice(), "Mode Register 1"),
        (b"MR2".as_slice(), "Mode Register 2"),
        (b"MR3".as_slice(), "Mode Register 3"),
        (b"LPDDR".as_slice(), "LPDDR Memory"),
    ];
    
    for (pattern, desc) in umc_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for UMC register addresses (AMD specific)
    println!("\n    {}", "Searching for UMC register patterns...".yellow());
    
    // UMC base addresses typically 0x50000, 0x150000 for channel 0/1
    let umc_bases: &[u32] = &[0x50000, 0x150000, 0x250000, 0x350000];
    for &base in umc_bases {
        let pattern = base.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if !matches.is_empty() {
            println!("    UMC Base 0x{:X}: {} references", base, matches.len());
        }
    }
    
    // Look for memory frequency values in MHz
    println!("\n    {}", "Memory frequency references:".yellow());
    let mem_freqs: &[u16] = &[2800, 2933, 3000, 3200, 3333, 3466, 3600, 3733, 3866, 4000];
    for &freq in mem_freqs {
        let pattern = freq.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if matches.len() > 0 && matches.len() < 100 {
            println!("    {} MHz: {} references", freq, matches.len());
        }
    }
}

fn analyze_all_fan_curves(data: &[u8]) {
    println!("\n{}", "  [FAN CURVES - DETAILED]".bold().bright_green());
    
    let mut fan_curves = Vec::new();
    
    // Look for fan curve patterns
    // Pattern 1: temp, speed pairs (both ascending)
    for i in 0..data.len().saturating_sub(32) {
        // Check for 8-point fan curve (16 bytes: 8 temps + 8 speeds interleaved)
        let chunk = &data[i..i+16];
        
        // Extract temps (even indices) and speeds (odd indices)
        let temps: Vec<u8> = (0..8).map(|j| chunk[j * 2]).collect();
        let speeds: Vec<u8> = (0..8).map(|j| chunk[j * 2 + 1]).collect();
        
        // Validate: temps should be 30-100, ascending; speeds 0-100 or 0-255
        let valid_temps = temps.windows(2).all(|w| w[0] <= w[1]) &&
                         temps.iter().all(|&t| (30..=105).contains(&t)) &&
                         temps[0] >= 35 && temps[7] <= 100;
        
        let valid_speeds = speeds.iter().all(|&s| s <= 100 || (s >= 200 && s <= 255));
        
        if valid_temps && valid_speeds {
            // Additional check: reasonable spread
            if temps[7] - temps[0] >= 20 {
                fan_curves.push((i, temps.clone(), speeds.clone()));
            }
        }
    }
    
    // Also look for separate temp and speed arrays
    for i in 0..data.len().saturating_sub(16) {
        let temps: Vec<u8> = data[i..i+8].to_vec();
        
        // Check if this looks like a temp array
        let valid_temps = temps.windows(2).all(|w| w[0] <= w[1]) &&
                         temps.iter().all(|&t| (30..=105).contains(&t)) &&
                         temps[0] >= 35 && temps[7] <= 100 &&
                         temps[7] - temps[0] >= 20;
        
        if valid_temps {
            // Look for corresponding speed array nearby
            for j in [i+8, i+16, i+32].iter() {
                if *j + 8 <= data.len() {
                    let speeds: Vec<u8> = data[*j..*j+8].to_vec();
                    let valid_speeds = speeds.iter().all(|&s| s <= 100);
                    if valid_speeds {
                        fan_curves.push((i, temps.clone(), speeds));
                        break;
                    }
                }
            }
        }
    }
    
    // Deduplicate and show
    fan_curves.sort_by_key(|(o, _, _)| *o);
    fan_curves.dedup_by_key(|(o, _, _)| *o);
    
    println!("    Found {} potential fan curves:", fan_curves.len());
    for (offset, temps, speeds) in fan_curves.iter().take(10) {
        println!("\n    @ 0x{:08X}:", offset);
        println!("      Temps:  {:?}°C", temps);
        println!("      Speeds: {:?}%", speeds);
        
        // Calculate curve characteristics
        let min_temp = temps[0];
        let max_temp = temps[temps.len()-1];
        let min_speed = speeds.iter().min().unwrap_or(&0);
        let max_speed = speeds.iter().max().unwrap_or(&100);
        println!("      Range: {}°C-{}°C → {}%-{}%", min_temp, max_temp, min_speed, max_speed);
    }
}


fn analyze_thermal_thresholds(data: &[u8]) {
    println!("\n{}", "  [THERMAL THRESHOLDS & THROTTLING]".bold().bright_green());
    
    // Known thermal limit values for AMD APUs
    let thermal_values: &[(u8, &str)] = &[
        (85, "Typical throttle start"),
        (90, "Heavy throttle"),
        (95, "Critical throttle"),
        (100, "Emergency shutdown warning"),
        (105, "Max Tj (junction temp)"),
    ];
    
    println!("    {}", "Searching for thermal limit structures...".yellow());
    
    // Look for thermal configuration structures
    // Pattern: multiple temperature thresholds in sequence
    let mut thermal_structs = Vec::new();
    
    for i in 0..data.len().saturating_sub(32) {
        // Look for ascending temperature sequences
        let chunk = &data[i..i+16];
        
        // Check for thermal table pattern
        let mut temps = Vec::new();
        for &b in chunk.iter() {
            if (40..=110).contains(&b) {
                temps.push(b);
            }
        }
        
        // Valid thermal table: 4+ temps, mostly ascending
        if temps.len() >= 4 {
            let ascending = temps.windows(2).filter(|w| w[0] <= w[1]).count();
            if ascending >= temps.len() - 2 {
                // Check for key values
                let has_85 = temps.contains(&85);
                let has_95 = temps.contains(&95);
                let has_100 = temps.contains(&100);
                
                if has_85 || has_95 || has_100 {
                    thermal_structs.push((i, temps.clone()));
                }
            }
        }
    }
    
    thermal_structs.sort_by_key(|(o, _)| *o);
    thermal_structs.dedup_by_key(|(o, _)| *o);
    
    println!("    Found {} thermal structures with key values:", thermal_structs.len());
    for (offset, temps) in thermal_structs.iter().take(15) {
        println!("      @ 0x{:08X}: {:?}°C", offset, temps);
    }
    
    // Search for specific throttle-related strings
    println!("\n    {}", "Throttle-related strings:".yellow());
    let throttle_patterns = [
        b"Throttle".as_slice(),
        b"PROCHOT".as_slice(),
        b"TjMax".as_slice(),
        b"Tctl".as_slice(),
        b"ThermalLimit".as_slice(),
        b"TempLimit".as_slice(),
        b"HotSpot".as_slice(),
        b"SkinTemp".as_slice(),
        b"STAPM".as_slice(),
    ];
    
    for pattern in throttle_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("      {}: {} @ {:?}", 
                String::from_utf8_lossy(pattern).green(),
                matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
}

fn analyze_smu_messages(data: &[u8]) {
    println!("\n{}", "  [SMU MESSAGE IDS]".bold().bright_green());
    
    // SMU messages are typically sent with specific IDs
    // Look for SMU message dispatch patterns
    
    // Known AMD SMU message IDs (Van Gogh/Aerith APU)
    let known_smu_msgs: &[(u8, &str)] = &[
        (0x01, "TestMessage"),
        (0x02, "GetSmuVersion"),
        (0x03, "GetDriverIfVersion"),
        (0x05, "SetAllowedFeaturesMaskLow"),
        (0x06, "SetAllowedFeaturesMaskHigh"),
        (0x07, "EnableAllSmuFeatures"),
        (0x08, "DisableAllSmuFeatures"),
        (0x09, "EnableSmuFeaturesLow"),
        (0x0A, "EnableSmuFeaturesHigh"),
        (0x0B, "DisableSmuFeaturesLow"),
        (0x0C, "DisableSmuFeaturesHigh"),
        (0x14, "SetHardMinGfxClk"),
        (0x15, "SetSoftMinGfxClk"),
        (0x16, "SetSoftMaxGfxClk"),
        (0x17, "SetHardMinFclkByFreq"),
        (0x18, "SetSoftMinFclk"),
        (0x19, "SetSoftMaxFclk"),
        (0x1A, "SetHardMinSocclkByFreq"),
        (0x1B, "SetSoftMinSocclk"),
        (0x1C, "SetSoftMaxSocclk"),
        (0x1D, "SetHardMinVcn"),
        (0x1E, "SetSoftMinVcn"),
        (0x1F, "SetSoftMaxVcn"),
        (0x23, "PowerDownVcn"),
        (0x24, "PowerUpVcn"),
        (0x25, "SetHardMinLclk"),
        (0x26, "SetSoftMinLclk"),
        (0x27, "SetSoftMaxLclk"),
        (0x2B, "SetFastPPTLimit"),
        (0x2C, "SetSlowPPTLimit"),
        (0x2D, "GetFastPPTLimit"),
        (0x2E, "GetSlowPPTLimit"),
        (0x2F, "GetGfxclkFrequency"),
        (0x30, "GetFclkFrequency"),
        (0x31, "GetCurrentVoltage"),
        (0x32, "GetCurrentPower"),
        (0x33, "GetCurrentTemperature"),
        (0x34, "GetAverageCpuActivity"),
        (0x35, "GetAverageGfxActivity"),
        (0x36, "SetDriverDramAddrHigh"),
        (0x37, "SetDriverDramAddrLow"),
        (0x38, "TransferTableSmu2Dram"),
        (0x39, "TransferTableDram2Smu"),
        (0x3A, "GetMetricsTable"),
        (0x3C, "SetSTAPMLimit"),
        (0x3D, "GetSTAPMLimit"),
        (0x3E, "SetTDCLimit"),
        (0x3F, "GetTDCLimit"),
        (0x40, "SetEDCLimit"),
        (0x41, "GetEDCLimit"),
        (0x42, "SetTHMLimit"),
        (0x43, "GetTHMLimit"),
    ];
    
    println!("    Known SMU Message IDs for Van Gogh/Aerith:");
    
    // Search for these message IDs in context
    for (id, name) in known_smu_msgs.iter() {
        // Look for the ID followed by typical SMU call patterns
        let mut found = false;
        for i in 0..data.len().saturating_sub(16) {
            if data[i] == *id {
                // Check context - look for SMU-related bytes nearby
                let ctx = &data[i.saturating_sub(4)..i.min(data.len()-8)+8];
                // SMU calls often have specific patterns
                if ctx.contains(&0x3B) || ctx.contains(&0x3C) { // MP1_SMN_C2PMSG
                    if !found {
                        println!("      0x{:02X}: {} - found in context", id, name.green());
                        found = true;
                    }
                }
            }
        }
    }
    
    // Look for SMU message dispatch function
    println!("\n    {}", "SMU dispatch patterns:".yellow());
    let smu_dispatch = find_pattern_all(data, b"SMU msg");
    for &offset in smu_dispatch.iter().take(5) {
        let ctx = get_context_string(data, offset, 100);
        println!("      @ 0x{:08X}: {}", offset, ctx.dimmed());
    }
}


fn analyze_power_tables_detailed(data: &[u8]) {
    println!("\n{}", "  [POWER TABLES - DETAILED]".bold().bright_green());
    
    // Power values in mW
    let power_values: &[(u32, &str)] = &[
        (3000, "3W - Ultra Low"),
        (4000, "4W - Min TDP"),
        (6000, "6W - Low"),
        (8000, "8W - Eco"),
        (10000, "10W - Medium Low"),
        (12000, "12W - Medium"),
        (15000, "15W - Default TDP"),
        (18000, "18W - High"),
        (20000, "20W - Performance"),
        (25000, "25W - Boost"),
        (28000, "28W - High Boost"),
        (30000, "30W - Max"),
    ];
    
    println!("    Power limit locations:");
    for (mw, desc) in power_values {
        let pattern = mw.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if !matches.is_empty() && matches.len() < 50 {
            println!("      {} ({}mW): {} locations", desc.green(), mw, matches.len());
            for &offset in matches.iter().take(4) {
                // Show context
                let ctx_start = offset.saturating_sub(8);
                let ctx_end = (offset + 16).min(data.len());
                let ctx = &data[ctx_start..ctx_end];
                println!("        @ 0x{:08X}: {}", offset, 
                    ctx.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
            }
        }
    }
    
    // Look for power table structures
    println!("\n    {}", "Power table structures:".yellow());
    
    // Pattern: multiple power values in sequence (STAPM, Fast PPT, Slow PPT, etc.)
    for i in 0..data.len().saturating_sub(32) {
        let mut cursor = Cursor::new(&data[i..i+32]);
        let mut vals = Vec::new();
        
        for _ in 0..8 {
            if let Ok(v) = cursor.read_u32::<LittleEndian>() {
                vals.push(v);
            }
        }
        
        // Check if this looks like a power table
        let power_vals: Vec<u32> = vals.iter()
            .filter(|&&v| v >= 1000 && v <= 50000 && v % 500 == 0)
            .copied()
            .collect();
        
        if power_vals.len() >= 3 {
            println!("      @ 0x{:08X}: {:?} mW", i, power_vals);
        }
    }
}

fn analyze_gpu_pstates(data: &[u8]) {
    println!("\n{}", "  [GPU P-STATES]".bold().bright_green());
    
    // GPU P-states typically contain: frequency, voltage pairs
    // Steam Deck GPU: 200-1600 MHz, 700-1200 mV
    
    println!("    {}", "Searching for GPU P-state tables...".yellow());
    
    let mut pstate_candidates = Vec::new();
    
    for i in 0..data.len().saturating_sub(64) {
        let mut cursor = Cursor::new(&data[i..i+64]);
        let mut entries = Vec::new();
        
        // Try to read freq/voltage pairs
        for _ in 0..8 {
            if let (Ok(freq), Ok(volt)) = (
                cursor.read_u32::<LittleEndian>(),
                cursor.read_u32::<LittleEndian>(),
            ) {
                // Valid GPU freq: 200-1800 MHz, Valid voltage: 600-1400 mV
                if (200..=1800).contains(&freq) && (600..=1400).contains(&volt) {
                    entries.push((freq, volt));
                }
            }
        }
        
        // Need at least 3 valid entries
        if entries.len() >= 3 {
            // Check if frequencies are ascending
            let freqs_ascending = entries.windows(2).all(|w| w[0].0 <= w[1].0);
            if freqs_ascending {
                pstate_candidates.push((i, entries));
            }
        }
    }
    
    println!("    Found {} potential P-state tables:", pstate_candidates.len());
    for (offset, entries) in pstate_candidates.iter().take(5) {
        println!("\n      @ 0x{:08X}:", offset);
        for (j, (freq, volt)) in entries.iter().enumerate() {
            println!("        P{}: {} MHz @ {} mV", j, freq, volt);
        }
    }
    
    // Also look for GFXCLK specific patterns
    println!("\n    {}", "GFXCLK references:".yellow());
    let gfx_patterns = [
        b"GfxClk".as_slice(),
        b"GFXCLK".as_slice(),
        b"gfxclk".as_slice(),
        b"GfxMaxFreq".as_slice(),
        b"GfxMinFreq".as_slice(),
    ];
    
    for pattern in gfx_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("      {}: {} matches", String::from_utf8_lossy(pattern).green(), matches.len());
        }
    }
}

fn analyze_apcb_apob(data: &[u8]) {
    println!("\n{}", "  [APCB/APOB STRUCTURES]".bold().bright_green());
    
    // APCB = AMD PSP Customization Block (input to PSP)
    // APOB = AMD PSP Output Block (output from PSP)
    
    let apcb_sig = b"APCB";
    let apob_sig = b"APOB";
    
    // Find APCB
    let apcb_matches = find_pattern_all(data, apcb_sig);
    println!("    APCB signatures: {} found", apcb_matches.len());
    for &offset in apcb_matches.iter().take(5) {
        if offset + 64 <= data.len() {
            let header = &data[offset..offset+64];
            println!("      @ 0x{:08X}:", offset);
            println!("        Header: {}", header[..16].iter()
                .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
            
            // Try to parse APCB header
            if header.len() >= 16 {
                let mut cursor = Cursor::new(&header[4..]);
                if let (Ok(size), Ok(version)) = (
                    cursor.read_u32::<LittleEndian>(),
                    cursor.read_u32::<LittleEndian>(),
                ) {
                    println!("        Size: 0x{:X}, Version: 0x{:X}", size, version);
                }
            }
        }
    }
    
    // Find APOB
    let apob_matches = find_pattern_all(data, apob_sig);
    println!("\n    APOB signatures: {} found", apob_matches.len());
    for &offset in apob_matches.iter().take(5) {
        println!("      @ 0x{:08X}", offset);
    }
    
    // Look for memory configuration in APCB
    println!("\n    {}", "Memory config patterns in APCB area:".yellow());
    let mem_patterns = [
        b"MemClkFreq".as_slice(),
        b"DimmConfig".as_slice(),
        b"SpdData".as_slice(),
        b"TimingMode".as_slice(),
        b"VddioMem".as_slice(),
        b"CadBus".as_slice(),
        b"DataBus".as_slice(),
    ];
    
    for pattern in mem_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("      {}: {} matches", String::from_utf8_lossy(pattern).green(), matches.len());
        }
    }
}

// Helper functions
fn find_pattern_all(data: &[u8], pattern: &[u8]) -> Vec<usize> {
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

fn get_context_string(data: &[u8], offset: usize, len: usize) -> String {
    let start = offset.saturating_sub(8);
    let end = (offset + len).min(data.len());
    data[start..end].iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
        .collect()
}
