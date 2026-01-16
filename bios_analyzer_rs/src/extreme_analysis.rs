//! Extreme deep analysis - CBS/PBS options, STAPM, PPT, hidden menus, voltage tables

use byteorder::{LittleEndian, ReadBytesExt};
use colored::Colorize;
use std::io::Cursor;

pub fn extreme_analysis(data: &[u8]) {
    println!("\n{}", "═".repeat(80).bright_magenta());
    println!("{}", " EXTREME DEEP ANALYSIS".bold().bright_magenta());
    println!("{}", "═".repeat(80).bright_magenta());

    // 1. CBS/PBS Menu structures
    analyze_cbs_pbs_menus(data);
    
    // 2. STAPM/PPT/TDC/EDC structures
    analyze_stapm_structures(data);
    
    // 3. Voltage regulation tables
    analyze_voltage_regulation(data);
    
    // 4. Memory training parameters
    analyze_memory_training(data);
    
    // 5. Clock generator / PLL settings
    analyze_pll_settings(data);
    
    // 6. Hidden UEFI variables
    analyze_hidden_variables(data);
    
    // 7. SMU firmware tables
    analyze_smu_tables(data);
    
    // 8. FCLK/UCLK/MCLK relationships
    analyze_clock_domains(data);
}


fn analyze_cbs_pbs_menus(data: &[u8]) {
    println!("\n{}", "  [CBS/PBS MENU STRUCTURES]".bold().bright_green());
    
    // CBS = Common BIOS Settings (AMD)
    // PBS = Platform BIOS Settings (OEM)
    
    let menu_patterns = [
        (b"CBS".as_slice(), "CBS Menu"),
        (b"PBS".as_slice(), "PBS Menu"),
        (b"NBIO".as_slice(), "NBIO Options"),
        (b"FCH".as_slice(), "FCH Options"),
        (b"DF ".as_slice(), "Data Fabric"),
        (b"UMC".as_slice(), "Memory Controller"),
        (b"SMU".as_slice(), "SMU Options"),
        (b"GNB".as_slice(), "Graphics North Bridge"),
        (b"DXIO".as_slice(), "DXIO Config"),
        (b"GMI".as_slice(), "Global Memory Interconnect"),
    ];
    
    for (pattern, desc) in menu_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() && matches.len() < 500 {
            println!("    {}: {} occurrences", desc.green(), matches.len());
        }
    }
    
    // Look for IFR (Internal Form Representation) structures
    println!("\n    {}", "IFR Form structures:".yellow());
    
    // IFR opcodes
    let ifr_form = [0x01u8]; // EFI_IFR_FORM_OP
    let ifr_one_of = [0x05u8]; // EFI_IFR_ONE_OF_OP
    let ifr_checkbox = [0x06u8]; // EFI_IFR_CHECKBOX_OP
    let ifr_numeric = [0x07u8]; // EFI_IFR_NUMERIC_OP
    
    let mut form_count = 0;
    let mut oneof_count = 0;
    let mut checkbox_count = 0;
    let mut numeric_count = 0;
    
    for i in 0..data.len().saturating_sub(8) {
        // IFR structures have specific header format
        if data[i] == 0x01 && data[i+1] >= 0x06 && data[i+1] <= 0x20 {
            form_count += 1;
        }
        if data[i] == 0x05 && data[i+1] >= 0x06 && data[i+1] <= 0x30 {
            oneof_count += 1;
        }
        if data[i] == 0x06 && data[i+1] >= 0x06 && data[i+1] <= 0x20 {
            checkbox_count += 1;
        }
        if data[i] == 0x07 && data[i+1] >= 0x06 && data[i+1] <= 0x30 {
            numeric_count += 1;
        }
    }
    
    println!("      Form opcodes: ~{}", form_count);
    println!("      OneOf opcodes: ~{}", oneof_count);
    println!("      Checkbox opcodes: ~{}", checkbox_count);
    println!("      Numeric opcodes: ~{}", numeric_count);
}


fn analyze_stapm_structures(data: &[u8]) {
    println!("\n{}", "  [STAPM/PPT/TDC/EDC STRUCTURES]".bold().bright_green());
    
    // STAPM = Skin Temperature Aware Power Management
    // PPT = Package Power Tracking (Fast/Slow)
    // TDC = Thermal Design Current
    // EDC = Electrical Design Current
    
    // Search for power management strings
    let pm_strings = [
        (b"STAPM".as_slice(), "STAPM Limit"),
        (b"FastPPT".as_slice(), "Fast PPT"),
        (b"SlowPPT".as_slice(), "Slow PPT"),
        (b"TDC".as_slice(), "TDC Limit"),
        (b"EDC".as_slice(), "EDC Limit"),
        (b"cTDP".as_slice(), "Configurable TDP"),
        (b"PL1".as_slice(), "Power Limit 1"),
        (b"PL2".as_slice(), "Power Limit 2"),
        (b"Tau".as_slice(), "Time constant"),
        (b"SkinTemp".as_slice(), "Skin Temperature"),
        (b"APU_PWR".as_slice(), "APU Power"),
        (b"SOC_PWR".as_slice(), "SOC Power"),
        (b"GFX_PWR".as_slice(), "GFX Power"),
    ];
    
    for (pattern, desc) in pm_strings {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for power limit structures (mW values)
    println!("\n    {}", "Power limit value patterns:".yellow());
    
    // Common TDP values for Steam Deck: 4W, 8W, 10W, 12W, 15W, 18W, 20W, 25W, 30W
    let tdp_values: &[(u32, &str)] = &[
        (4000, "4W Min"),
        (8000, "8W Eco"),
        (10000, "10W Low"),
        (12000, "12W Medium"),
        (15000, "15W Default"),
        (18000, "18W High"),
        (20000, "20W Perf"),
        (25000, "25W Boost"),
        (30000, "30W Max"),
    ];
    
    for (mw, desc) in tdp_values {
        let pattern = mw.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if !matches.is_empty() && matches.len() < 100 {
            // Check context for power-related structures
            let mut valid_matches = Vec::new();
            for &offset in &matches {
                if offset >= 4 && offset + 16 < data.len() {
                    // Check if surrounded by other power values
                    let ctx = &data[offset-4..offset+16];
                    let has_power_context = ctx.windows(4).any(|w| {
                        let v = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
                        v >= 1000 && v <= 50000 && v % 1000 == 0
                    });
                    if has_power_context {
                        valid_matches.push(offset);
                    }
                }
            }
            if !valid_matches.is_empty() {
                println!("      {}: {} valid @ {:?}", desc, valid_matches.len(),
                    valid_matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
            }
        }
    }
}


fn analyze_voltage_regulation(data: &[u8]) {
    println!("\n{}", "  [VOLTAGE REGULATION TABLES]".bold().bright_green());
    
    // VRM (Voltage Regulator Module) related
    let vrm_patterns = [
        (b"VDDCR".as_slice(), "VDDCR (CPU/SOC)"),
        (b"VDDG".as_slice(), "VDDG (Infinity Fabric)"),
        (b"VDDP".as_slice(), "VDDP (PHY)"),
        (b"VDD18".as_slice(), "VDD18 (1.8V)"),
        (b"VDDIO".as_slice(), "VDDIO (Memory I/O)"),
        (b"VDDQ".as_slice(), "VDDQ (Memory Data)"),
        (b"VDD2".as_slice(), "VDD2 (Memory Core)"),
        (b"MVDD".as_slice(), "MVDD (Memory)"),
        (b"SVI2".as_slice(), "SVI2 Interface"),
        (b"SVI3".as_slice(), "SVI3 Interface"),
        (b"VID".as_slice(), "Voltage ID"),
        (b"LoadLine".as_slice(), "Load Line"),
        (b"Droop".as_slice(), "Voltage Droop"),
    ];
    
    for (pattern, desc) in vrm_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() && matches.len() < 200 {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
    
    // Look for voltage tables (mV values)
    println!("\n    {}", "Voltage value patterns:".yellow());
    
    // Common voltages: 500mV, 750mV, 800mV, 900mV, 1000mV, 1050mV, 1100mV, 1150mV, 1200mV
    let voltage_values: &[(u16, &str)] = &[
        (500, "0.5V VDDQ"),
        (750, "0.75V"),
        (800, "0.8V"),
        (900, "0.9V"),
        (1000, "1.0V"),
        (1050, "1.05V VDD2"),
        (1100, "1.1V"),
        (1150, "1.15V"),
        (1200, "1.2V"),
        (1350, "1.35V"),
        (1500, "1.5V"),
        (1800, "1.8V VDD1"),
    ];
    
    for (mv, desc) in voltage_values {
        let pattern = mv.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        // Filter to reasonable count
        if matches.len() > 5 && matches.len() < 500 {
            println!("      {} ({}mV): {} refs", desc, mv, matches.len());
        }
    }
    
    // Look for voltage offset tables
    println!("\n    {}", "Searching for voltage offset structures...".yellow());
    let mut offset_tables = Vec::new();
    
    for i in 0..data.len().saturating_sub(32) {
        // Voltage offsets are typically small signed values (-200 to +200 mV)
        let chunk = &data[i..i+16];
        let mut offsets = Vec::new();
        
        for j in (0..16).step_by(2) {
            let val = i16::from_le_bytes([chunk[j], chunk[j+1]]);
            if val >= -200 && val <= 200 {
                offsets.push(val);
            }
        }
        
        // Valid offset table: 4+ values, mix of positive/negative
        if offsets.len() >= 4 {
            let has_neg = offsets.iter().any(|&v| v < 0);
            let has_pos = offsets.iter().any(|&v| v > 0);
            let has_zero = offsets.iter().any(|&v| v == 0);
            
            if (has_neg || has_pos) && has_zero {
                offset_tables.push((i, offsets.clone()));
            }
        }
    }
    
    offset_tables.sort_by_key(|(o, _)| *o);
    offset_tables.dedup_by_key(|(o, _)| *o);
    
    println!("      Found {} potential voltage offset tables", offset_tables.len());
    for (offset, vals) in offset_tables.iter().take(5) {
        println!("        @ 0x{:08X}: {:?} mV", offset, vals);
    }
}


fn analyze_memory_training(data: &[u8]) {
    println!("\n{}", "  [MEMORY TRAINING PARAMETERS]".bold().bright_green());
    
    // Memory training related strings
    let training_patterns = [
        (b"MemTrain".as_slice(), "Memory Training"),
        (b"DqsTrain".as_slice(), "DQS Training"),
        (b"WrLvl".as_slice(), "Write Leveling"),
        (b"RdDqs".as_slice(), "Read DQS"),
        (b"WrDqs".as_slice(), "Write DQS"),
        (b"RxEn".as_slice(), "Receiver Enable"),
        (b"TxDq".as_slice(), "TX DQ"),
        (b"Vref".as_slice(), "Voltage Reference"),
        (b"2D Train".as_slice(), "2D Training"),
        (b"PMU".as_slice(), "PHY Management Unit"),
        (b"DRAM Init".as_slice(), "DRAM Init"),
        (b"MR Write".as_slice(), "Mode Register Write"),
        (b"ZQ Cal".as_slice(), "ZQ Calibration"),
        (b"CA Train".as_slice(), "Command/Address Training"),
        (b"CS Train".as_slice(), "Chip Select Training"),
    ];
    
    for (pattern, desc) in training_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(2).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for training result structures
    println!("\n    {}", "Training delay values:".yellow());
    
    // Training delays are typically 0-63 or 0-127 (6-7 bit values)
    let mut delay_tables = Vec::new();
    
    for i in 0..data.len().saturating_sub(32) {
        let chunk = &data[i..i+16];
        
        // Check for delay table pattern (values 0-63, mostly non-zero)
        let valid_delays = chunk.iter().filter(|&&b| b <= 63 && b > 0).count();
        let zeros = chunk.iter().filter(|&&b| b == 0).count();
        
        if valid_delays >= 12 && zeros <= 4 {
            // Check for reasonable spread
            let min = chunk.iter().filter(|&&b| b <= 63).min().unwrap_or(&0);
            let max = chunk.iter().filter(|&&b| b <= 63).max().unwrap_or(&63);
            
            if max - min >= 10 && max - min <= 50 {
                delay_tables.push((i, chunk.to_vec()));
            }
        }
    }
    
    delay_tables.sort_by_key(|(o, _)| *o);
    delay_tables.dedup_by_key(|(o, _)| *o);
    
    println!("      Found {} potential delay tables", delay_tables.len());
    for (offset, vals) in delay_tables.iter().take(5) {
        println!("        @ 0x{:08X}: {:?}", offset, &vals[..8.min(vals.len())]);
    }
}

fn analyze_pll_settings(data: &[u8]) {
    println!("\n{}", "  [PLL / CLOCK GENERATOR]".bold().bright_green());
    
    // PLL related patterns
    let pll_patterns = [
        (b"PLL".as_slice(), "PLL Reference"),
        (b"DPLL".as_slice(), "Digital PLL"),
        (b"APLL".as_slice(), "Analog PLL"),
        (b"MPLL".as_slice(), "Memory PLL"),
        (b"GPLL".as_slice(), "Graphics PLL"),
        (b"CPLL".as_slice(), "Core PLL"),
        (b"RefClk".as_slice(), "Reference Clock"),
        (b"BCLK".as_slice(), "Base Clock"),
        (b"Spread".as_slice(), "Spread Spectrum"),
        (b"SSC".as_slice(), "Spread Spectrum Clock"),
        (b"ClkGen".as_slice(), "Clock Generator"),
        (b"DFS".as_slice(), "Digital Frequency Synthesizer"),
    ];
    
    for (pattern, desc) in pll_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() && matches.len() < 200 {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
    
    // Look for frequency multiplier/divider tables
    println!("\n    {}", "Frequency divider patterns:".yellow());
    
    // Common reference: 100MHz, dividers/multipliers
    let ref_100mhz = 100000000u32.to_le_bytes();
    let ref_100mhz_matches = find_pattern_all(data, &ref_100mhz);
    if !ref_100mhz_matches.is_empty() {
        println!("      100MHz reference: {} @ {:?}", ref_100mhz_matches.len(),
            ref_100mhz_matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
    }
    
    // Look for spread spectrum settings (typically 0.1% - 2.0%)
    println!("\n    {}", "Spread spectrum candidates:".yellow());
    let ss_values: &[(u16, &str)] = &[
        (10, "0.1%"),
        (25, "0.25%"),
        (50, "0.5%"),
        (100, "1.0%"),
        (150, "1.5%"),
        (200, "2.0%"),
    ];
    
    for (val, desc) in ss_values {
        let pattern = val.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if matches.len() > 10 && matches.len() < 1000 {
            println!("      {} spread: {} refs", desc, matches.len());
        }
    }
}


fn analyze_hidden_variables(data: &[u8]) {
    println!("\n{}", "  [HIDDEN UEFI VARIABLES]".bold().bright_green());
    
    // NVRAM variable patterns
    let var_patterns = [
        (b"Setup".as_slice(), "Setup Variable"),
        (b"PlatformConfig".as_slice(), "Platform Config"),
        (b"AmdSetup".as_slice(), "AMD Setup"),
        (b"CbsSetup".as_slice(), "CBS Setup"),
        (b"PbsSetup".as_slice(), "PBS Setup"),
        (b"MemoryConfig".as_slice(), "Memory Config"),
        (b"PerfTune".as_slice(), "Performance Tuning"),
        (b"OcConfig".as_slice(), "Overclock Config"),
        (b"FanConfig".as_slice(), "Fan Config"),
        (b"ThermalConfig".as_slice(), "Thermal Config"),
        (b"PowerConfig".as_slice(), "Power Config"),
    ];
    
    for (pattern, desc) in var_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() && matches.len() < 100 {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for NVRAM variable headers (EFI_VARIABLE_HEADER pattern)
    println!("\n    {}", "NVRAM structure patterns:".yellow());
    
    // Variable attributes (common values)
    // 0x07 = NV + BS + RT (Non-Volatile, Boot Service, Runtime)
    // 0x03 = NV + BS
    let mut nvram_candidates = Vec::new();
    
    for i in 0..data.len().saturating_sub(64) {
        // Look for variable header pattern
        if data[i] == 0x07 && data[i+1] == 0x00 && data[i+2] == 0x00 && data[i+3] == 0x00 {
            // Check for reasonable size field
            if i + 8 < data.len() {
                let size = u32::from_le_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]);
                if size > 0 && size < 0x10000 {
                    nvram_candidates.push((i, size));
                }
            }
        }
    }
    
    println!("      Found {} potential NVRAM headers", nvram_candidates.len());
}

fn analyze_smu_tables(data: &[u8]) {
    println!("\n{}", "  [SMU FIRMWARE TABLES]".bold().bright_green());
    
    // SMU table signatures
    let smu_patterns = [
        (b"SMU_".as_slice(), "SMU Prefix"),
        (b"PMFW".as_slice(), "Power Management FW"),
        (b"SMUFW".as_slice(), "SMU Firmware"),
        (b"MP1_".as_slice(), "MP1 (SMU)"),
        (b"MP2_".as_slice(), "MP2 (Sensor Hub)"),
        (b"RSMU".as_slice(), "RSMU"),
        (b"BIOS2SMU".as_slice(), "BIOS to SMU"),
        (b"SMU2BIOS".as_slice(), "SMU to BIOS"),
        (b"DpmTable".as_slice(), "DPM Table"),
        (b"PptTable".as_slice(), "PPT Table"),
        (b"SmuMetrics".as_slice(), "SMU Metrics"),
    ];
    
    for (pattern, desc) in smu_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for SMU message response codes
    println!("\n    {}", "SMU response patterns:".yellow());
    
    // SMU_RESULT values
    let smu_results: &[(u8, &str)] = &[
        (0x01, "SMU_Result_OK"),
        (0xFE, "SMU_Result_Failed"),
        (0xFD, "SMU_Result_UnknownCmd"),
        (0xFC, "SMU_Result_CmdRejectedPrereq"),
        (0xFB, "SMU_Result_CmdRejectedBusy"),
    ];
    
    for (code, desc) in smu_results {
        // Look for code in context of SMU handling
        let mut count = 0;
        for i in 0..data.len().saturating_sub(16) {
            if data[i] == *code {
                // Check for SMU-related context
                let start = i.saturating_sub(32);
                let end = (i + 32).min(data.len());
                let ctx = &data[start..end];
                if ctx.windows(3).any(|w| w == b"SMU" || w == b"smu") {
                    count += 1;
                }
            }
        }
        if count > 0 {
            println!("      {}: {} in SMU context", desc, count);
        }
    }
}


fn analyze_clock_domains(data: &[u8]) {
    println!("\n{}", "  [CLOCK DOMAINS - FCLK/UCLK/MCLK]".bold().bright_green());
    
    // AMD clock domain strings
    let clock_patterns = [
        (b"FCLK".as_slice(), "Infinity Fabric Clock"),
        (b"UCLK".as_slice(), "Unified Memory Clock"),
        (b"MCLK".as_slice(), "Memory Clock"),
        (b"GFXCLK".as_slice(), "Graphics Clock"),
        (b"SOCCLK".as_slice(), "SOC Clock"),
        (b"LCLK".as_slice(), "Link Clock"),
        (b"DCLK".as_slice(), "Display Clock"),
        (b"VCLK".as_slice(), "Video Clock"),
        (b"DCFCLK".as_slice(), "Display Controller Fabric"),
        (b"DISPCLK".as_slice(), "Display Clock"),
        (b"PHYCLK".as_slice(), "PHY Clock"),
        (b"REFCLK".as_slice(), "Reference Clock"),
    ];
    
    for (pattern, desc) in clock_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for clock frequency tables (MHz values for Van Gogh)
    println!("\n    {}", "Clock frequency values:".yellow());
    
    // Van Gogh/Aerith typical frequencies
    let freq_values: &[(u16, &str)] = &[
        (400, "400 MHz (FCLK min)"),
        (800, "800 MHz"),
        (933, "933 MHz"),
        (1067, "1067 MHz"),
        (1200, "1200 MHz"),
        (1333, "1333 MHz"),
        (1467, "1467 MHz"),
        (1600, "1600 MHz (FCLK max)"),
        (1800, "1800 MHz (GFXCLK)"),
        (2133, "2133 MHz"),
        (2400, "2400 MHz"),
        (2667, "2667 MHz"),
        (2800, "2800 MHz (MCLK)"),
        (3200, "3200 MHz (MCLK max)"),
        (3466, "3466 MHz"),
        (3600, "3600 MHz"),
    ];
    
    for (mhz, desc) in freq_values {
        let pattern = mhz.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if matches.len() > 5 && matches.len() < 500 {
            println!("      {}: {} refs", desc, matches.len());
        }
    }
    
    // Look for FCLK:UCLK ratio structures
    println!("\n    {}", "FCLK:UCLK ratio patterns:".yellow());
    
    // Common ratios: 1:1, 1:2, 2:1
    let mut ratio_candidates = Vec::new();
    
    for i in 0..data.len().saturating_sub(16) {
        // Look for ratio structure: [numerator, denominator] pairs
        if data[i] == 1 && data[i+1] == 0 && data[i+2] == 1 && data[i+3] == 0 {
            // 1:1 ratio
            ratio_candidates.push((i, "1:1"));
        }
        if data[i] == 1 && data[i+1] == 0 && data[i+2] == 2 && data[i+3] == 0 {
            // 1:2 ratio
            ratio_candidates.push((i, "1:2"));
        }
        if data[i] == 2 && data[i+1] == 0 && data[i+2] == 1 && data[i+3] == 0 {
            // 2:1 ratio
            ratio_candidates.push((i, "2:1"));
        }
    }
    
    // Deduplicate nearby entries
    ratio_candidates.sort_by_key(|(o, _)| *o);
    let mut filtered = Vec::new();
    let mut last_offset = 0usize;
    for (offset, ratio) in ratio_candidates {
        if offset > last_offset + 16 {
            filtered.push((offset, ratio));
            last_offset = offset;
        }
    }
    
    println!("      Found {} potential ratio structures", filtered.len());
    for (offset, ratio) in filtered.iter().take(10) {
        println!("        @ 0x{:08X}: {}", offset, ratio);
    }
}

// Helper function
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
