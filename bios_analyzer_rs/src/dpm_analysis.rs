//! DPM (Dynamic Power Management) table analysis for Van Gogh/Aerith APU

use colored::Colorize;

pub fn analyze_dpm_tables(data: &[u8]) {
    println!("\n{}", "═".repeat(80).bright_cyan());
    println!("{}", " DPM TABLE ANALYSIS".bold().bright_cyan());
    println!("{}", "═".repeat(80).bright_cyan());

    // 1. DPM State tables
    analyze_dpm_states(data);
    
    // 2. Workload profiles
    analyze_workload_profiles(data);
    
    // 3. Power Play tables
    analyze_powerplay(data);
    
    // 4. Soft limits
    analyze_soft_limits(data);
}

fn analyze_dpm_states(data: &[u8]) {
    println!("\n{}", "  [DPM STATE TABLES]".bold().bright_green());
    
    let dpm_patterns = [
        (b"DPM".as_slice(), "DPM Reference"),
        (b"DpmLevel".as_slice(), "DPM Level"),
        (b"DpmState".as_slice(), "DPM State"),
        (b"DpmFreq".as_slice(), "DPM Frequency"),
        (b"DpmVolt".as_slice(), "DPM Voltage"),
        (b"GfxDpm".as_slice(), "GFX DPM"),
        (b"SocDpm".as_slice(), "SOC DPM"),
        (b"FclkDpm".as_slice(), "FCLK DPM"),
        (b"UclkDpm".as_slice(), "UCLK DPM"),
        (b"VclkDpm".as_slice(), "VCLK DPM"),
        (b"DclkDpm".as_slice(), "DCLK DPM"),
    ];
    
    for (pattern, desc) in dpm_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() && matches.len() < 100 {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
}


fn analyze_workload_profiles(data: &[u8]) {
    println!("\n{}", "  [WORKLOAD PROFILES]".bold().bright_green());
    
    let profile_patterns = [
        (b"Workload".as_slice(), "Workload"),
        (b"Profile".as_slice(), "Profile"),
        (b"Gaming".as_slice(), "Gaming Mode"),
        (b"Power Saver".as_slice(), "Power Saver"),
        (b"Balanced".as_slice(), "Balanced"),
        (b"Performance".as_slice(), "Performance"),
        (b"Custom".as_slice(), "Custom"),
        (b"Turbo".as_slice(), "Turbo"),
        (b"Silent".as_slice(), "Silent"),
        (b"Battery".as_slice(), "Battery"),
    ];
    
    for (pattern, desc) in profile_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() && matches.len() < 50 {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
}

fn analyze_powerplay(data: &[u8]) {
    println!("\n{}", "  [POWERPLAY TABLES]".bold().bright_green());
    
    // PowerPlay table signatures
    let pp_patterns = [
        (b"PowerPlay".as_slice(), "PowerPlay"),
        (b"PPTable".as_slice(), "PP Table"),
        (b"SoftMax".as_slice(), "Soft Max"),
        (b"SoftMin".as_slice(), "Soft Min"),
        (b"HardMax".as_slice(), "Hard Max"),
        (b"HardMin".as_slice(), "Hard Min"),
        (b"BoostFreq".as_slice(), "Boost Frequency"),
        (b"BaseFreq".as_slice(), "Base Frequency"),
    ];
    
    for (pattern, desc) in pp_patterns {
        let matches = find_pattern_all(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for frequency/voltage pairs that could be PowerPlay entries
    println!("\n    {}", "Searching for freq/volt pairs...".yellow());
    
    let mut pp_candidates = Vec::new();
    for i in 0..data.len().saturating_sub(16) {
        // PowerPlay entry format: [freq_mhz:u16, volt_mv:u16] or similar
        let freq = u16::from_le_bytes([data[i], data[i+1]]);
        let volt = u16::from_le_bytes([data[i+2], data[i+3]]);
        
        // Valid GPU freq: 200-1800 MHz, Valid voltage: 600-1400 mV
        if (200..=1800).contains(&freq) && (600..=1400).contains(&volt) {
            // Check for multiple consecutive entries
            let freq2 = u16::from_le_bytes([data[i+4], data[i+5]]);
            let volt2 = u16::from_le_bytes([data[i+6], data[i+7]]);
            
            if (200..=1800).contains(&freq2) && (600..=1400).contains(&volt2) {
                if freq2 > freq { // Ascending frequencies
                    pp_candidates.push((i, vec![(freq, volt), (freq2, volt2)]));
                }
            }
        }
    }
    
    // Deduplicate
    pp_candidates.sort_by_key(|(o, _)| *o);
    let mut filtered = Vec::new();
    let mut last = 0usize;
    for (offset, entries) in pp_candidates {
        if offset > last + 8 {
            filtered.push((offset, entries));
            last = offset;
        }
    }
    
    println!("    Found {} potential PowerPlay entries", filtered.len());
    for (offset, entries) in filtered.iter().take(10) {
        println!("      @ 0x{:08X}: {:?}", offset, entries);
    }
}

fn analyze_soft_limits(data: &[u8]) {
    println!("\n{}", "  [SOFT/HARD LIMITS]".bold().bright_green());
    
    // Look for limit structures
    let limit_patterns = [
        (b"Limit".as_slice(), "Limit"),
        (b"Max".as_slice(), "Max"),
        (b"Min".as_slice(), "Min"),
        (b"Cap".as_slice(), "Cap"),
        (b"Ceiling".as_slice(), "Ceiling"),
        (b"Floor".as_slice(), "Floor"),
    ];
    
    for (pattern, desc) in limit_patterns {
        let matches = find_pattern_all(data, pattern);
        if matches.len() > 10 && matches.len() < 500 {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
    
    // Search for specific limit values
    println!("\n    {}", "GPU frequency limits:".yellow());
    
    let gpu_limits: &[(u16, &str)] = &[
        (200, "200 MHz (min)"),
        (400, "400 MHz"),
        (800, "800 MHz"),
        (1100, "1100 MHz"),
        (1300, "1300 MHz"),
        (1600, "1600 MHz (max)"),
        (1800, "1800 MHz (boost)"),
    ];
    
    for (mhz, desc) in gpu_limits {
        let pattern = mhz.to_le_bytes();
        let matches = find_pattern_all(data, &pattern);
        if matches.len() > 5 && matches.len() < 300 {
            println!("      {}: {} refs", desc, matches.len());
        }
    }
}

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
