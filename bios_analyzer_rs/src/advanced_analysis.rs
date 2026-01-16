//! Advanced BIOS analysis - hidden features, SMU commands, optimization options

use byteorder::{LittleEndian, ReadBytesExt};
use colored::Colorize;
use std::collections::HashMap;
use std::io::Cursor;

/// Search for all interesting strings and functions
pub fn find_hidden_features(data: &[u8]) {
    println!("\n{}", "═".repeat(80).bright_magenta());
    println!("{}", " HIDDEN FEATURES & OPTIMIZATION SEARCH".bold().bright_magenta());
    println!("{}", "═".repeat(80).bright_magenta());

    // 1. SMU Commands and Messages
    find_smu_commands(data);
    
    // 2. CBS/PBS Menu Options
    find_cbs_pbs_options(data);
    
    // 3. Performance/Power profiles
    find_performance_profiles(data);
    
    // 4. Hidden frequency options
    find_hidden_frequencies(data);
    
    // 5. Thermal management
    find_thermal_management(data);
    
    // 6. Fan control
    find_fan_control(data);
    
    // 7. Display/Refresh rate
    find_display_options(data);
    
    // 8. Battery/Power management
    find_battery_options(data);
    
    // 9. Debug/Developer options
    find_debug_options(data);
    
    // 10. AMD specific features
    find_amd_features(data);
}

fn find_smu_commands(data: &[u8]) {
    println!("\n{}", "  [SMU COMMANDS & MESSAGES]".bold().cyan());
    
    // Known SMU message patterns
    let smu_patterns: &[(&[u8], &str)] = &[
        (b"SetHardMin", "GPU Hard Min Clock"),
        (b"SetSoftMin", "GPU Soft Min Clock"),
        (b"SetSoftMax", "GPU Soft Max Clock"),
        (b"SetHardMax", "GPU Hard Max Clock"),
        (b"PPT", "Package Power Tracking"),
        (b"STAPM", "Skin Temp Aware Power Management"),
        (b"TDC", "Thermal Design Current"),
        (b"EDC", "Electrical Design Current"),
        (b"THM", "Thermal"),
        (b"FAN", "Fan Control"),
        (b"GfxClk", "Graphics Clock"),
        (b"SocClk", "SoC Clock"),
        (b"FClk", "Fabric Clock"),
        (b"UClk", "Unified Memory Clock"),
        (b"VClk", "Video Clock"),
        (b"DClk", "Display Clock"),
        (b"PowerLimit", "Power Limit"),
        (b"TempLimit", "Temperature Limit"),
        (b"CurrentLimit", "Current Limit"),
    ];
    
    for (pattern, desc) in smu_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {} [{}]: {} matches", desc.green(), 
                String::from_utf8_lossy(pattern), matches.len());
            for offset in matches.iter().take(3) {
                // Get context
                let start = offset.saturating_sub(16);
                let end = (offset + 48).min(data.len());
                let ctx = &data[start..end];
                let ctx_str: String = ctx.iter()
                    .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                    .collect();
                println!("      @ 0x{:08X}: {}", offset, ctx_str.dimmed());
            }
        }
    }
}


fn find_cbs_pbs_options(data: &[u8]) {
    println!("\n{}", "  [CBS/PBS MENU OPTIONS]".bold().cyan());
    
    // CBS = Common BIOS Settings, PBS = Platform BIOS Settings
    let menu_patterns: &[(&[u8], &str)] = &[
        (b"Memory Clock", "Memory Frequency Setting"),
        (b"Infinity Fabric", "IF/FCLK Setting"),
        (b"Core Performance", "CPU Performance"),
        (b"Power Supply", "Power Settings"),
        (b"Thermal Control", "Thermal Management"),
        (b"Fan Control", "Fan Settings"),
        (b"CPU Voltage", "CPU Voltage Offset"),
        (b"SOC Voltage", "SoC Voltage"),
        (b"GFX Voltage", "GPU Voltage"),
        (b"VDDCR", "Voltage Rail"),
        (b"VDDIO", "I/O Voltage"),
        (b"VDDP", "PHY Voltage"),
        (b"cTDP", "Configurable TDP"),
        (b"PBO", "Precision Boost Overdrive"),
        (b"Curve Optimizer", "Voltage Curve"),
        (b"Core Count", "Active Cores"),
        (b"SMT", "Simultaneous Multithreading"),
        (b"Boost Override", "Boost Clock Override"),
        (b"CPPC", "Collaborative Power Performance"),
        (b"C-State", "CPU Power States"),
        (b"Package Power", "Package TDP"),
        (b"PROCHOT", "Processor Hot"),
    ];
    
    for (pattern, desc) in menu_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {} [{}]: {} matches", desc.green(), 
                String::from_utf8_lossy(pattern), matches.len());
        }
    }
    
    // Also search UTF-16LE versions
    println!("    {}", "Searching UTF-16LE strings...".dimmed());
    let utf16_patterns = [
        "Memory", "Clock", "Voltage", "Power", "Thermal", "Fan",
        "Boost", "Performance", "Frequency", "Speed", "Limit",
    ];
    
    for pattern in utf16_patterns {
        let utf16: Vec<u8> = pattern.encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        let matches = find_all_patterns(data, &utf16);
        if !matches.is_empty() {
            println!("      {} (UTF16): {} matches", pattern.yellow(), matches.len());
        }
    }
}

fn find_performance_profiles(data: &[u8]) {
    println!("\n{}", "  [PERFORMANCE PROFILES]".bold().cyan());
    
    let profile_patterns: &[(&[u8], &str)] = &[
        (b"Performance", "Performance Mode"),
        (b"Balanced", "Balanced Mode"),
        (b"Power Saver", "Power Saving Mode"),
        (b"Silent", "Silent Mode"),
        (b"Turbo", "Turbo Mode"),
        (b"Gaming", "Gaming Mode"),
        (b"Battery", "Battery Mode"),
        (b"AC Power", "AC Power Mode"),
        (b"Plugged", "Plugged In Mode"),
        (b"Unplugged", "Unplugged Mode"),
        (b"Max Performance", "Maximum Performance"),
        (b"Quiet", "Quiet Mode"),
    ];
    
    for (pattern, desc) in profile_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
}

fn find_hidden_frequencies(data: &[u8]) {
    println!("\n{}", "  [HIDDEN FREQUENCY OPTIONS]".bold().cyan());
    
    // Look for MHz/GHz strings
    let freq_patterns: &[(&[u8], &str)] = &[
        (b"MHz", "Frequency in MHz"),
        (b"GHz", "Frequency in GHz"),
        (b"MT/s", "Memory Transfer Rate"),
        (b"Mbps", "Megabits per second"),
        (b"3200", "3200 MHz"),
        (b"3600", "3600 MHz"),
        (b"4000", "4000 MHz"),
        (b"6400", "6400 MT/s"),
        (b"7200", "7200 MT/s"),
        (b"1600", "1600 MHz GPU"),
        (b"2000", "2000 MHz"),
        (b"2400", "2400 MHz"),
    ];
    
    for (pattern, desc) in freq_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
    
    // Look for frequency value tables (sequential numbers)
    println!("    {}", "Searching for frequency tables...".dimmed());
    let mut freq_tables = Vec::new();
    
    for i in 0..data.len().saturating_sub(32) {
        // Look for patterns like: 2800, 2900, 3000, 3100, 3200 (in various formats)
        let mut cursor = Cursor::new(&data[i..i+20]);
        if let (Ok(v1), Ok(v2), Ok(v3), Ok(v4), Ok(v5)) = (
            cursor.read_u16::<LittleEndian>(),
            cursor.read_u16::<LittleEndian>(),
            cursor.read_u16::<LittleEndian>(),
            cursor.read_u16::<LittleEndian>(),
            cursor.read_u16::<LittleEndian>(),
        ) {
            // Check if sequential and in reasonable range
            if (2000..=4000).contains(&v1) && 
               (2000..=4000).contains(&v5) &&
               v2 > v1 && v3 > v2 && v4 > v3 && v5 > v4 &&
               (v2 - v1) < 200 && (v3 - v2) < 200 {
                freq_tables.push((i, vec![v1, v2, v3, v4, v5]));
            }
        }
    }
    
    for (offset, vals) in freq_tables.iter().take(5) {
        println!("      @ 0x{:08X}: {:?} MHz", offset, vals);
    }
}


fn find_thermal_management(data: &[u8]) {
    println!("\n{}", "  [THERMAL MANAGEMENT]".bold().cyan());
    
    let thermal_patterns: &[(&[u8], &str)] = &[
        (b"Thermal", "Thermal Control"),
        (b"Temperature", "Temperature Setting"),
        (b"Throttle", "Throttling"),
        (b"TjMax", "Max Junction Temp"),
        (b"Tctl", "Control Temperature"),
        (b"Tdie", "Die Temperature"),
        (b"Hotspot", "Hotspot Temperature"),
        (b"Skin Temp", "Skin Temperature"),
        (b"APU Temp", "APU Temperature"),
        (b"GPU Temp", "GPU Temperature"),
        (b"PROCHOT", "Processor Hot Signal"),
        (b"Cooling", "Cooling Control"),
        (b"Heat", "Heat Management"),
    ];
    
    for (pattern, desc) in thermal_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} matches", desc.green(), matches.len());
            // Show first match context
            if let Some(&offset) = matches.first() {
                let ctx = get_string_context(data, offset, 64);
                println!("      @ 0x{:08X}: {}", offset, ctx.dimmed());
            }
        }
    }
    
    // Look for temperature values (typical range 40-105°C)
    println!("    {}", "Searching for temperature thresholds...".dimmed());
    let temp_values: &[u8] = &[40, 50, 60, 70, 75, 80, 85, 90, 95, 100, 105];
    
    for &temp in temp_values {
        // Look for temp followed by related bytes
        for i in 0..data.len().saturating_sub(8) {
            if data[i] == temp {
                // Check if it looks like a temp table (ascending values)
                let next_bytes = &data[i..i+8];
                if next_bytes.windows(2).all(|w| w[0] <= w[1]) &&
                   next_bytes.iter().all(|&b| (30..=110).contains(&b)) {
                    println!("      Temp table @ 0x{:08X}: {:?}°C", i, 
                        next_bytes.iter().map(|&b| b as i32).collect::<Vec<_>>());
                    break;
                }
            }
        }
    }
}

fn find_fan_control(data: &[u8]) {
    println!("\n{}", "  [FAN CONTROL]".bold().cyan());
    
    let fan_patterns: &[(&[u8], &str)] = &[
        (b"Fan", "Fan Control"),
        (b"FAN", "Fan Control (caps)"),
        (b"RPM", "Fan Speed RPM"),
        (b"PWM", "PWM Control"),
        (b"Duty", "Duty Cycle"),
        (b"Speed", "Fan Speed"),
        (b"Curve", "Fan Curve"),
        (b"Auto", "Auto Fan"),
        (b"Manual", "Manual Fan"),
        (b"Silent", "Silent Fan Mode"),
        (b"Cool", "Cooling"),
    ];
    
    for (pattern, desc) in fan_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
    
    // Look for fan curve data (temp -> speed pairs)
    println!("    {}", "Searching for fan curves...".dimmed());
    for i in 0..data.len().saturating_sub(16) {
        // Pattern: temp1, speed1, temp2, speed2, ... (both ascending)
        let chunk = &data[i..i+16];
        let temps: Vec<u8> = (0..8).step_by(2).map(|j| chunk[j]).collect();
        let speeds: Vec<u8> = (1..8).step_by(2).map(|j| chunk[j]).collect();
        
        // Valid fan curve: temps 30-100, speeds 0-100 or 0-255
        let valid_temps = temps.windows(2).all(|w| w[0] < w[1]) &&
                         temps.iter().all(|&t| (30..=100).contains(&t));
        let valid_speeds = speeds.windows(2).all(|w| w[0] <= w[1]) &&
                          speeds.iter().all(|&s| s <= 100 || s == 255);
        
        if valid_temps && valid_speeds && temps[0] >= 35 && temps[3] <= 95 {
            println!("      Fan curve @ 0x{:08X}:", i);
            println!("        Temps:  {:?}°C", temps);
            println!("        Speeds: {:?}%", speeds);
            break; // Just show first one
        }
    }
}

fn find_display_options(data: &[u8]) {
    println!("\n{}", "  [DISPLAY OPTIONS]".bold().cyan());
    
    let display_patterns: &[(&[u8], &str)] = &[
        (b"Refresh", "Refresh Rate"),
        (b"60Hz", "60Hz Mode"),
        (b"90Hz", "90Hz Mode"),
        (b"120Hz", "120Hz Mode"),
        (b"VRR", "Variable Refresh Rate"),
        (b"FreeSync", "AMD FreeSync"),
        (b"Resolution", "Display Resolution"),
        (b"1280x800", "Native LCD Resolution"),
        (b"1920x1080", "1080p"),
        (b"720p", "720p Mode"),
        (b"Panel", "Display Panel"),
        (b"eDP", "Embedded DisplayPort"),
        (b"Backlight", "Backlight Control"),
        (b"Brightness", "Brightness"),
        (b"HDR", "High Dynamic Range"),
        (b"Gamma", "Gamma Correction"),
    ];
    
    for (pattern, desc) in display_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
}


fn find_battery_options(data: &[u8]) {
    println!("\n{}", "  [BATTERY & POWER OPTIONS]".bold().cyan());
    
    let battery_patterns: &[(&[u8], &str)] = &[
        (b"Battery", "Battery Settings"),
        (b"Charge", "Charging Control"),
        (b"Discharge", "Discharge Settings"),
        (b"AC Adapter", "AC Power"),
        (b"DC Power", "DC/Battery Power"),
        (b"Power State", "Power States"),
        (b"Sleep", "Sleep Mode"),
        (b"Hibernate", "Hibernate"),
        (b"Suspend", "Suspend Mode"),
        (b"Wake", "Wake Settings"),
        (b"S0", "S0 State"),
        (b"S3", "S3 Sleep"),
        (b"S4", "S4 Hibernate"),
        (b"S5", "S5 Soft Off"),
        (b"Modern Standby", "Modern Standby"),
        (b"Connected Standby", "Connected Standby"),
    ];
    
    for (pattern, desc) in battery_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} matches", desc.green(), matches.len());
        }
    }
}

fn find_debug_options(data: &[u8]) {
    println!("\n{}", "  [DEBUG & DEVELOPER OPTIONS]".bold().cyan());
    
    let debug_patterns: &[(&[u8], &str)] = &[
        (b"Debug", "Debug Mode"),
        (b"Verbose", "Verbose Output"),
        (b"Serial", "Serial Console"),
        (b"UART", "UART Debug"),
        (b"Log", "Logging"),
        (b"Trace", "Trace Mode"),
        (b"Assert", "Assertions"),
        (b"Test", "Test Mode"),
        (b"Factory", "Factory Mode"),
        (b"Engineering", "Engineering Mode"),
        (b"Developer", "Developer Mode"),
        (b"Unlock", "Unlock Feature"),
        (b"Hidden", "Hidden Option"),
        (b"Secret", "Secret Option"),
        (b"Override", "Override Setting"),
        (b"Force", "Force Option"),
        (b"Bypass", "Bypass Check"),
        (b"Disable", "Disable Feature"),
        (b"Enable", "Enable Feature"),
    ];
    
    for (pattern, desc) in debug_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            println!("    {}: {} matches", desc.green(), matches.len());
            // Show context for interesting ones
            if pattern == b"Hidden" || pattern == b"Secret" || pattern == b"Unlock" {
                for &offset in matches.iter().take(3) {
                    let ctx = get_string_context(data, offset, 48);
                    println!("      @ 0x{:08X}: {}", offset, ctx.dimmed());
                }
            }
        }
    }
}

fn find_amd_features(data: &[u8]) {
    println!("\n{}", "  [AMD SPECIFIC FEATURES]".bold().cyan());
    
    let amd_patterns: &[(&[u8], &str)] = &[
        // SMU Features
        (b"SMU", "System Management Unit"),
        (b"DXIO", "DXIO PHY"),
        (b"GMI", "Global Memory Interconnect"),
        (b"xGMI", "Extended GMI"),
        (b"WAFL", "WAFL Link"),
        
        // Power Management
        (b"STAPM", "Skin Temp Aware PM"),
        (b"PPT", "Package Power Tracking"),
        (b"TDC", "Thermal Design Current"),
        (b"EDC", "Electrical Design Current"),
        (b"CCLK", "Core Clock"),
        (b"GFXCLK", "Graphics Clock"),
        (b"SOCCLK", "SoC Clock"),
        (b"FCLK", "Fabric Clock"),
        (b"UCLK", "Unified Memory Clock"),
        (b"MCLK", "Memory Clock"),
        (b"VCLK", "Video Clock"),
        (b"DCLK", "Display Clock"),
        (b"LCLK", "Link Clock"),
        
        // Voltage
        (b"VDDCR_VDD", "Core Voltage"),
        (b"VDDCR_SOC", "SoC Voltage"),
        (b"VDDCR_GFX", "GFX Voltage"),
        (b"VDDIO_MEM", "Memory I/O Voltage"),
        (b"VDD_MEM", "Memory Voltage"),
        
        // Features
        (b"PBO", "Precision Boost Overdrive"),
        (b"CPB", "Core Performance Boost"),
        (b"CPPC", "Collaborative Power/Perf"),
        (b"PSS", "Performance Supported States"),
        (b"CST", "C-States"),
        (b"CC6", "Core C6 State"),
        (b"PC6", "Package C6 State"),
        (b"DF", "Data Fabric"),
        (b"UMC", "Unified Memory Controller"),
        (b"GMC", "Graphics Memory Controller"),
        (b"NBIO", "North Bridge I/O"),
        (b"FCH", "Fusion Controller Hub"),
        (b"PSP", "Platform Security Processor"),
        (b"MP1", "Management Processor 1"),
        (b"MP2", "Management Processor 2"),
        
        // AGESA
        (b"AGESA", "AMD Generic Encapsulated SW Arch"),
        (b"ABL", "AGESA Boot Loader"),
        (b"APCB", "AMD PSP Customization Block"),
        (b"APOB", "AMD PSP Output Block"),
        
        // Memory
        (b"LPDDR5", "LPDDR5 Memory"),
        (b"DDR5", "DDR5 Memory"),
        (b"PHY", "Memory PHY"),
        (b"DQS", "Data Strobe"),
        (b"CA", "Command/Address"),
        (b"Training", "Memory Training"),
    ];
    
    let mut found_features: Vec<(&str, usize, Vec<usize>)> = Vec::new();
    
    for (pattern, desc) in amd_patterns {
        let matches = find_all_patterns(data, pattern);
        if !matches.is_empty() {
            found_features.push((desc, matches.len(), matches));
        }
    }
    
    // Sort by count
    found_features.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (desc, count, offsets) in found_features.iter().take(30) {
        println!("    {}: {} matches", desc.green(), count);
        if *count <= 5 {
            for offset in offsets {
                println!("      @ 0x{:08X}", offset);
            }
        }
    }
}

// Helper functions
fn find_all_patterns(data: &[u8], pattern: &[u8]) -> Vec<usize> {
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

fn get_string_context(data: &[u8], offset: usize, len: usize) -> String {
    let start = offset.saturating_sub(8);
    let end = (offset + len).min(data.len());
    data[start..end].iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
        .collect()
}
