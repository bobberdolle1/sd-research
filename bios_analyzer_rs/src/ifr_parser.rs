//! IFR (Internal Form Representation) parser - find hidden BIOS menu options

use colored::Colorize;

pub fn parse_ifr_options(data: &[u8]) {
    println!("\n{}", "═".repeat(80).bright_magenta());
    println!("{}", " IFR HIDDEN OPTIONS PARSER".bold().bright_magenta());
    println!("{}", "═".repeat(80).bright_magenta());

    // Search for interesting option strings
    find_fclk_options(data);
    find_spread_spectrum_options(data);
    find_memory_ratio_options(data);
    find_power_options(data);
    find_hidden_frequency_options(data);
}

fn find_fclk_options(data: &[u8]) {
    println!("\n{}", "  [FCLK OPTIONS]".bold().bright_green());
    
    // Search for FCLK related strings
    let patterns = [
        b"FCLK".as_slice(),
        b"Fclk".as_slice(),
        b"fclk".as_slice(),
        b"FabricClk".as_slice(),
        b"Fabric Clock".as_slice(),
        b"InfinityFabric".as_slice(),
        b"DF Clock".as_slice(),
        b"DfClk".as_slice(),
    ];
    
    for pattern in patterns {
        for i in 0..data.len().saturating_sub(pattern.len() + 50) {
            if &data[i..i+pattern.len()] == pattern {
                // Get context around the match
                let start = i.saturating_sub(20);
                let end = (i + pattern.len() + 80).min(data.len());
                let ctx = &data[start..end];
                
                // Check if this looks like a menu option (has printable chars)
                let printable: String = ctx.iter()
                    .filter(|&&b| b >= 0x20 && b <= 0x7E)
                    .map(|&b| b as char)
                    .collect();
                
                if printable.len() > 15 {
                    println!("    @ 0x{:08X}: {}", i, printable.dimmed());
                }
            }
        }
    }
    
    // Look for FCLK frequency values with context
    println!("\n    {}", "FCLK frequency values with context:".yellow());
    let fclk_freqs: &[u16] = &[1600, 1733, 1800, 1867, 1900, 2000];
    
    for &freq in fclk_freqs {
        let pattern = freq.to_le_bytes();
        for i in 0..data.len().saturating_sub(20) {
            if data[i] == pattern[0] && data[i+1] == pattern[1] {
                // Check context for FCLK-related bytes
                let ctx_start = i.saturating_sub(16);
                let ctx_end = (i + 16).min(data.len());
                let ctx = &data[ctx_start..ctx_end];
                
                // Look for other frequency values nearby (indicates freq table)
                let has_other_freq = ctx.windows(2).any(|w| {
                    let v = u16::from_le_bytes([w[0], w[1]]);
                    v >= 1400 && v <= 2100 && v != freq
                });
                
                if has_other_freq {
                    let hex: String = ctx.iter().map(|b| format!("{:02X} ", b)).collect();
                    println!("      {} MHz @ 0x{:08X}: {}", freq, i, hex.dimmed());
                }
            }
        }
    }
}


fn find_spread_spectrum_options(data: &[u8]) {
    println!("\n{}", "  [SPREAD SPECTRUM OPTIONS]".bold().bright_green());
    
    let patterns = [
        b"Spread".as_slice(),
        b"SSC".as_slice(),
        b"SpreadSpectrum".as_slice(),
        b"Spread Spectrum".as_slice(),
    ];
    
    let mut found = Vec::new();
    
    for pattern in patterns {
        for i in 0..data.len().saturating_sub(pattern.len() + 50) {
            if &data[i..i+pattern.len()] == pattern {
                let start = i.saturating_sub(10);
                let end = (i + pattern.len() + 60).min(data.len());
                let ctx = &data[start..end];
                
                let printable: String = ctx.iter()
                    .filter(|&&b| b >= 0x20 && b <= 0x7E)
                    .map(|&b| b as char)
                    .collect();
                
                if printable.len() > 10 && !found.contains(&i) {
                    found.push(i);
                    println!("    @ 0x{:08X}: {}", i, printable.dimmed());
                }
            }
        }
    }
}

fn find_memory_ratio_options(data: &[u8]) {
    println!("\n{}", "  [MEMORY RATIO / FCLK:UCLK OPTIONS]".bold().bright_green());
    
    let patterns = [
        b"UCLK".as_slice(),
        b"Uclk".as_slice(),
        b"UclkDiv".as_slice(),
        b"MemClk".as_slice(),
        b"Memory Clock".as_slice(),
        b"MCLK".as_slice(),
        b"Ratio".as_slice(),
        b"1:1".as_slice(),
        b"2:1".as_slice(),
        b"Auto".as_slice(),
        b"Sync".as_slice(),
        b"Async".as_slice(),
    ];
    
    for pattern in patterns {
        let mut count = 0;
        for i in 0..data.len().saturating_sub(pattern.len()) {
            if &data[i..i+pattern.len()] == pattern {
                count += 1;
                if count <= 3 {
                    let start = i.saturating_sub(10);
                    let end = (i + pattern.len() + 50).min(data.len());
                    let ctx = &data[start..end];
                    
                    let printable: String = ctx.iter()
                        .filter(|&&b| b >= 0x20 && b <= 0x7E)
                        .map(|&b| b as char)
                        .collect();
                    
                    if printable.len() > 8 {
                        println!("    {} @ 0x{:08X}: {}", 
                            String::from_utf8_lossy(pattern).green(),
                            i, printable.dimmed());
                    }
                }
            }
        }
        if count > 3 {
            println!("    {} : {} total matches", String::from_utf8_lossy(pattern).green(), count);
        }
    }
}

fn find_power_options(data: &[u8]) {
    println!("\n{}", "  [HIDDEN POWER OPTIONS]".bold().bright_green());
    
    let patterns = [
        b"PPT".as_slice(),
        b"STAPM".as_slice(),
        b"TDC".as_slice(),
        b"EDC".as_slice(),
        b"cTDP".as_slice(),
        b"PowerLimit".as_slice(),
        b"Power Limit".as_slice(),
        b"TDP".as_slice(),
        b"Watt".as_slice(),
    ];
    
    for pattern in patterns {
        let mut matches = Vec::new();
        for i in 0..data.len().saturating_sub(pattern.len()) {
            if &data[i..i+pattern.len()] == pattern {
                matches.push(i);
            }
        }
        
        if !matches.is_empty() && matches.len() < 50 {
            println!("    {} : {} matches", String::from_utf8_lossy(pattern).green(), matches.len());
            for &offset in matches.iter().take(3) {
                let start = offset.saturating_sub(10);
                let end = (offset + pattern.len() + 40).min(data.len());
                let ctx = &data[start..end];
                
                let printable: String = ctx.iter()
                    .filter(|&&b| b >= 0x20 && b <= 0x7E)
                    .map(|&b| b as char)
                    .collect();
                
                if printable.len() > 10 {
                    println!("      @ 0x{:08X}: {}", offset, printable.dimmed());
                }
            }
        }
    }
}

fn find_hidden_frequency_options(data: &[u8]) {
    println!("\n{}", "  [HIDDEN FREQUENCY OPTIONS > 3200 MHz]".bold().bright_green());
    
    // Look for memory frequencies above 3200 MHz (6400 MT/s)
    let high_freqs: &[(u16, &str)] = &[
        (3266, "3266 MHz (~6533 MT/s)"),
        (3333, "3333 MHz (~6666 MT/s)"),
        (3400, "3400 MHz (~6800 MT/s)"),
        (3466, "3466 MHz (~6933 MT/s)"),
        (3533, "3533 MHz (~7066 MT/s)"),
        (3600, "3600 MHz (~7200 MT/s)"),
        (3733, "3733 MHz (~7466 MT/s)"),
        (3866, "3866 MHz (~7733 MT/s)"),
        (4000, "4000 MHz (~8000 MT/s)"),
    ];
    
    for (freq, desc) in high_freqs {
        let pattern = freq.to_le_bytes();
        let mut matches = Vec::new();
        
        for i in 0..data.len().saturating_sub(2) {
            if data[i] == pattern[0] && data[i+1] == pattern[1] {
                matches.push(i);
            }
        }
        
        if !matches.is_empty() && matches.len() < 200 {
            println!("    {} : {} refs", desc.green(), matches.len());
            
            // Show first few with context
            for &offset in matches.iter().take(2) {
                let start = offset.saturating_sub(8);
                let end = (offset + 16).min(data.len());
                let hex: String = data[start..end].iter()
                    .map(|b| format!("{:02X} ", b))
                    .collect();
                println!("      @ 0x{:08X}: {}", offset, hex.dimmed());
            }
        }
    }
    
    // Look for frequency table patterns (consecutive MHz values)
    println!("\n    {}", "Frequency tables with high values:".yellow());
    
    for i in 0..data.len().saturating_sub(32) {
        let mut freqs = Vec::new();
        for j in (0..32).step_by(2) {
            let val = u16::from_le_bytes([data[i+j], data[i+j+1]]);
            if val >= 2800 && val <= 4200 {
                freqs.push(val);
            }
        }
        
        // Valid freq table: 4+ values, ascending
        if freqs.len() >= 4 {
            let ascending = freqs.windows(2).all(|w| w[0] <= w[1]);
            let has_high = freqs.iter().any(|&f| f > 3200);
            
            if ascending && has_high {
                println!("      @ 0x{:08X}: {:?} MHz", i, freqs);
            }
        }
    }
}
