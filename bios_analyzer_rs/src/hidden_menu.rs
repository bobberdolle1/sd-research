//! Hidden menu options search - find CBS/PBS options not exposed by SREP

use colored::Colorize;

pub fn find_hidden_menus(data: &[u8]) {
    println!("\n{}", "═".repeat(80).bright_yellow());
    println!("{}", " HIDDEN MENU OPTIONS SEARCH".bold().bright_yellow());
    println!("{}", "═".repeat(80).bright_yellow());

    // 1. Memory related options
    find_memory_options(data);
    
    // 2. Power/Performance options  
    find_power_options(data);
    
    // 3. Debug/Developer options
    find_debug_options(data);
    
    // 4. Clock/PLL options
    find_clock_options(data);
}

fn find_memory_options(data: &[u8]) {
    println!("\n{}", "  [MEMORY OPTIONS]".bold().bright_green());
    
    let patterns = [
        // Timing related
        (b"Trfc".as_slice(), "tRFC (Refresh Cycle)"),
        (b"Trefi".as_slice(), "tREFI (Refresh Interval)"),
        (b"Tfaw".as_slice(), "tFAW (Four Activate Window)"),
        (b"Trrd".as_slice(), "tRRD (Row-to-Row Delay)"),
        (b"Twtr".as_slice(), "tWTR (Write-to-Read)"),
        (b"Trtp".as_slice(), "tRTP (Read-to-Precharge)"),
        (b"Twr ".as_slice(), "tWR (Write Recovery)"),
        (b"Tcwl".as_slice(), "tCWL (CAS Write Latency)"),
        (b"Txp".as_slice(), "tXP (Exit Power Down)"),
        
        // Controller options
        (b"BankGroup".as_slice(), "Bank Group Swap"),
        (b"Interleav".as_slice(), "Interleaving"),
        (b"Scrambler".as_slice(), "Memory Scrambler"),
        (b"PowerDown".as_slice(), "Power Down Mode"),
        (b"SelfRefresh".as_slice(), "Self Refresh"),
        (b"GearDown".as_slice(), "Gear Down Mode"),
        (b"CmdRate".as_slice(), "Command Rate (1T/2T)"),
        (b"AddrCmd".as_slice(), "Address Command Parity"),
        
        // PHY/Training
        (b"RttNom".as_slice(), "RTT Nominal"),
        (b"RttWr".as_slice(), "RTT Write"),
        (b"RttPark".as_slice(), "RTT Park"),
        (b"DqDrv".as_slice(), "DQ Drive Strength"),
        (b"CaDrv".as_slice(), "CA Drive Strength"),
        (b"CkDrv".as_slice(), "CK Drive Strength"),
        (b"CsDrv".as_slice(), "CS Drive Strength"),
        (b"OdtDrv".as_slice(), "ODT Drive Strength"),
        (b"Vref".as_slice(), "Voltage Reference"),
        (b"DqsOffset".as_slice(), "DQS Offset"),
        (b"RxOffset".as_slice(), "RX Offset"),
        (b"TxOffset".as_slice(), "TX Offset"),
    ];
    
    for (pattern, desc) in patterns {
        let matches = find_all(data, pattern);
        if !matches.is_empty() && matches.len() < 50 {
            println!("    {} : {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
}


fn find_power_options(data: &[u8]) {
    println!("\n{}", "  [POWER/PERFORMANCE OPTIONS]".bold().bright_green());
    
    let patterns = [
        // Power limits
        (b"STAPM".as_slice(), "STAPM Limit"),
        (b"FastPPT".as_slice(), "Fast PPT"),
        (b"SlowPPT".as_slice(), "Slow PPT"),
        (b"FPPT".as_slice(), "Fast PPT Alt"),
        (b"SPPT".as_slice(), "Slow PPT Alt"),
        (b"TDC".as_slice(), "TDC Limit"),
        (b"EDC".as_slice(), "EDC Limit"),
        (b"THM".as_slice(), "Thermal Limit"),
        (b"cTDP".as_slice(), "Configurable TDP"),
        (b"PBP".as_slice(), "Processor Base Power"),
        (b"MTP".as_slice(), "Max Turbo Power"),
        
        // CPU/APU
        (b"CoreCof".as_slice(), "Core COF (Frequency)"),
        (b"CpuVid".as_slice(), "CPU VID"),
        (b"CpuFid".as_slice(), "CPU FID"),
        (b"CpuDid".as_slice(), "CPU DID"),
        (b"BoostFmax".as_slice(), "Boost Fmax"),
        (b"PState".as_slice(), "P-State"),
        (b"CState".as_slice(), "C-State"),
        (b"CC6".as_slice(), "CC6 (Core C6)"),
        (b"PC6".as_slice(), "PC6 (Package C6)"),
        (b"CPB".as_slice(), "Core Performance Boost"),
        (b"SmuFeature".as_slice(), "SMU Feature"),
        
        // GPU
        (b"GfxClk".as_slice(), "GFX Clock"),
        (b"GfxVid".as_slice(), "GFX VID"),
        (b"iGPU".as_slice(), "iGPU Setting"),
        (b"UmaSize".as_slice(), "UMA Frame Buffer"),
        (b"UmaAbove".as_slice(), "UMA Above 4G"),
        
        // Fabric
        (b"FclkFreq".as_slice(), "FCLK Frequency"),
        (b"UclkDiv".as_slice(), "UCLK Divider"),
        (b"DfCstate".as_slice(), "DF C-State"),
    ];
    
    for (pattern, desc) in patterns {
        let matches = find_all(data, pattern);
        if !matches.is_empty() && matches.len() < 100 {
            println!("    {} : {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
}

fn find_debug_options(data: &[u8]) {
    println!("\n{}", "  [DEBUG/HIDDEN OPTIONS]".bold().bright_green());
    
    let patterns = [
        (b"DebugEn".as_slice(), "Debug Enable"),
        (b"SerialDebug".as_slice(), "Serial Debug"),
        (b"Verbose".as_slice(), "Verbose Mode"),
        (b"FactoryMode".as_slice(), "Factory Mode"),
        (b"EngineerMode".as_slice(), "Engineer Mode"),
        (b"TestMode".as_slice(), "Test Mode"),
        (b"Unlock".as_slice(), "Unlock"),
        (b"Hidden".as_slice(), "Hidden"),
        (b"Advanced".as_slice(), "Advanced"),
        (b"Expert".as_slice(), "Expert"),
        (b"Override".as_slice(), "Override"),
        (b"Force".as_slice(), "Force"),
        (b"Bypass".as_slice(), "Bypass"),
        (b"Disable".as_slice(), "Disable Check"),
        (b"SkipCheck".as_slice(), "Skip Check"),
        (b"NoLimit".as_slice(), "No Limit"),
    ];
    
    for (pattern, desc) in patterns {
        let matches = find_all(data, pattern);
        if matches.len() > 0 && matches.len() < 50 {
            println!("    {} : {}", desc.green(), matches.len());
        }
    }
}

fn find_clock_options(data: &[u8]) {
    println!("\n{}", "  [CLOCK/PLL OPTIONS]".bold().bright_green());
    
    let patterns = [
        (b"SpreadSpectrum".as_slice(), "Spread Spectrum"),
        (b"SSC".as_slice(), "SSC (Spread Spectrum)"),
        (b"ClockGating".as_slice(), "Clock Gating"),
        (b"PowerGating".as_slice(), "Power Gating"),
        (b"DeepSleep".as_slice(), "Deep Sleep"),
        (b"CLDO".as_slice(), "CLDO (Clock LDO)"),
        (b"PllLock".as_slice(), "PLL Lock"),
        (b"RefClk".as_slice(), "Reference Clock"),
        (b"BCLK".as_slice(), "Base Clock"),
        (b"SocClk".as_slice(), "SOC Clock"),
        (b"DfClk".as_slice(), "DF Clock"),
        (b"VclkDclk".as_slice(), "VCLK/DCLK"),
    ];
    
    for (pattern, desc) in patterns {
        let matches = find_all(data, pattern);
        if !matches.is_empty() && matches.len() < 100 {
            println!("    {} : {} @ {:?}", desc.green(), matches.len(),
                matches.iter().take(3).map(|o| format!("0x{:X}", o)).collect::<Vec<_>>());
        }
    }
    
    // Look for specific interesting values
    println!("\n    {}", "Interesting frequency values:".yellow());
    
    // FCLK values (MHz as u16)
    let fclk_vals: &[(u16, &str)] = &[
        (1600, "1600 MHz FCLK"),
        (1733, "1733 MHz FCLK"),
        (1800, "1800 MHz FCLK"),
        (1867, "1867 MHz FCLK"),
        (1900, "1900 MHz FCLK"),
        (2000, "2000 MHz FCLK"),
    ];
    
    for (val, desc) in fclk_vals {
        let pattern = val.to_le_bytes();
        let matches = find_all(data, &pattern);
        if matches.len() > 5 && matches.len() < 200 {
            println!("      {} : {} refs", desc, matches.len());
        }
    }
}

fn find_all(data: &[u8], pattern: &[u8]) -> Vec<usize> {
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
