//! Steam Deck BIOS Deep Analyzer
//! Полный реверс-инжиниринг F7A BIOS

use colored::Colorize;
use memmap2::Mmap;
use std::fs::File;

mod structures;
mod patterns;
mod analysis;
mod deep_analysis;
mod advanced_analysis;
mod ultra_deep;
mod extreme_analysis;
mod dpm_analysis;
mod hidden_menu;
mod ifr_parser;

use structures::*;
use analysis::*;
use deep_analysis::*;
use advanced_analysis::*;
use ultra_deep::*;
use extreme_analysis::*;
use dpm_analysis::*;
use hidden_menu::*;
use ifr_parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("F7A0133_sign.fd");
    
    println!("{}", "═".repeat(80).cyan());
    println!("{}", " STEAM DECK BIOS DEEP ANALYZER v0.1".bold().cyan());
    println!("{}", "═".repeat(80).cyan());
    
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data = &mmap[..];
    
    println!("\n{}: {}", "File".bold(), filename);
    println!("{}: {} bytes ({:.2} MB)", "Size".bold(), data.len(), data.len() as f64 / 1024.0 / 1024.0);
    
    let mut report = BiosReport::new(filename, data.len());
    
    // 1. UEFI Volume Analysis
    analyze_uefi_volumes(data, &mut report);
    
    // 2. SPD Structures
    analyze_spd_structures(data, &mut report);
    
    // 3. Frequency Tables
    analyze_frequency_tables(data, &mut report);
    
    // 4. Power Management
    analyze_power_management(data, &mut report);
    
    // 5. SMU Firmware
    analyze_smu(data, &mut report);

    // 6. String Analysis
    analyze_strings(data, &mut report);
    
    // 7. GUID Analysis
    analyze_guids(data, &mut report);
    
    // 8. Numeric Tables
    analyze_numeric_tables(data, &mut report);
    
    // 9. AMD/PSP Structures
    analyze_amd_psp(data, &mut report);
    
    // 10. EC Firmware
    analyze_ec(data, &mut report);
    
    // 11. Deep Analysis (GPU, Voltages, Timings, etc.)
    let deep_report = deep_analyze(data);
    deep_report.print();
    
    // 12. Advanced Analysis (Hidden features, SMU commands, etc.)
    find_hidden_features(data);
    
    // 13. Ultra Deep Analysis (H2O unlock, UMC, Fan curves, Thermal, SMU IDs)
    ultra_deep_analysis(data);
    
    // 14. Extreme Analysis (CBS/PBS, STAPM, Voltages, Clock domains)
    extreme_analysis(data);
    
    // 15. DPM Table Analysis
    analyze_dpm_tables(data);
    
    // 16. Hidden Menu Options
    find_hidden_menus(data);
    
    // 17. IFR Parser - Hidden Options
    parse_ifr_options(data);
    
    // Print Report
    report.print();
    
    // Save JSON
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write("bios_analysis_report.json", &json)?;
    println!("\n{}", "Report saved to bios_analysis_report.json".green());
    
    Ok(())
}
