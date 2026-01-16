//! Deep analysis functions for advanced BIOS structures

use crate::structures::*;
use byteorder::{LittleEndian, ReadBytesExt};
use colored::Colorize;
use std::io::Cursor;

/// Extended analysis - GPU, Voltages, Timings, etc.
pub fn deep_analyze(data: &[u8]) -> DeepAnalysisReport {
    let mut report = DeepAnalysisReport::default();
    
    println!("\n{}", "═".repeat(80).magenta());
    println!("{}", " DEEP ANALYSIS".bold().magenta());
    println!("{}", "═".repeat(80).magenta());
    
    // GPU Clock analysis
    analyze_gpu_clocks(data, &mut report);
    
    // Voltage tables
    analyze_voltage_tables(data, &mut report);
    
    // Memory timing structures
    analyze_memory_timings(data, &mut report);
    
    // Fan curves
    analyze_fan_curves(data, &mut report);
    
    // Display/Panel info
    analyze_display(data, &mut report);
    
    // ACPI tables
    analyze_acpi(data, &mut report);
    
    // Boot configuration
    analyze_boot_config(data, &mut report);
    
    report
}

#[derive(Default, Debug)]
pub struct DeepAnalysisReport {
    pub gpu_clocks: Vec<GpuClockEntry>,
    pub voltage_tables: Vec<VoltageTable>,
    pub memory_timings: Vec<MemoryTiming>,
    pub fan_curves: Vec<FanCurve>,
    pub display_info: Vec<DisplayInfo>,
    pub acpi_tables: Vec<AcpiTable>,
    pub boot_entries: Vec<BootEntry>,
}

#[derive(Debug)]
pub struct GpuClockEntry {
    pub offset: u64,
    pub min_mhz: u32,
    pub max_mhz: u32,
    pub default_mhz: u32,
}

#[derive(Debug)]
pub struct VoltageTable {
    pub offset: u64,
    pub voltage_type: String,
    pub values_mv: Vec<u32>,
}

#[derive(Debug)]
pub struct MemoryTiming {
    pub offset: u64,
    pub tcl: u8,
    pub trcd: u8,
    pub trp: u8,
    pub tras: u8,
}


#[derive(Debug)]
pub struct FanCurve {
    pub offset: u64,
    pub temp_points: Vec<u8>,
    pub speed_points: Vec<u8>,
}

#[derive(Debug)]
pub struct DisplayInfo {
    pub offset: u64,
    pub panel_type: String,
    pub resolution: String,
}

#[derive(Debug)]
pub struct AcpiTable {
    pub offset: u64,
    pub signature: String,
    pub size: u32,
}

#[derive(Debug)]
pub struct BootEntry {
    pub offset: u64,
    pub description: String,
}

fn analyze_gpu_clocks(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing GPU clocks...".dimmed());
    
    // Steam Deck GPU (RDNA2): 200-1600 MHz typical range
    // Look for patterns like: min_clock, max_clock, default_clock
    
    // Search for common GPU clock values
    let gpu_freqs = [200u32, 400, 800, 1000, 1100, 1200, 1300, 1400, 1500, 1600];
    
    for freq in gpu_freqs {
        let pattern = freq.to_le_bytes();
        let mut i = 0;
        while i < data.len() - 4 {
            if &data[i..i+4] == pattern {
                // Check if this looks like a GPU clock structure
                if i + 12 <= data.len() {
                    let mut cursor = Cursor::new(&data[i..i+12]);
                    if let (Ok(v1), Ok(v2), Ok(v3)) = (
                        cursor.read_u32::<LittleEndian>(),
                        cursor.read_u32::<LittleEndian>(),
                        cursor.read_u32::<LittleEndian>(),
                    ) {
                        // All three should be in GPU freq range
                        if (200..=1800).contains(&v1) && 
                           (200..=1800).contains(&v2) && 
                           (200..=1800).contains(&v3) {
                            report.gpu_clocks.push(GpuClockEntry {
                                offset: i as u64,
                                min_mhz: v1.min(v2).min(v3),
                                max_mhz: v1.max(v2).max(v3),
                                default_mhz: v2,
                            });
                        }
                    }
                }
            }
            i += 4;
        }
    }
    
    // Deduplicate
    report.gpu_clocks.sort_by_key(|e| e.offset);
    report.gpu_clocks.dedup_by_key(|e| e.offset);
    
    println!("    Found {} potential GPU clock entries", report.gpu_clocks.len());
}

fn analyze_voltage_tables(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing voltage tables...".dimmed());
    
    // Look for voltage values in mV (800-1400 range typical)
    // VDDCR_SOC, VDDCR_GFX, VDDIO_MEM, etc.
    
    let voltage_markers = [
        (800u32, "VDDCR_SOC min"),
        (900, "VDDCR typical"),
        (1000, "VDDCR mid"),
        (1050, "VDD2 default"),
        (1100, "VDDCR high"),
        (1200, "VDDCR max"),
    ];
    
    for (mv, desc) in voltage_markers {
        let pattern = mv.to_le_bytes();
        for (i, window) in data.windows(4).enumerate() {
            if window == pattern {
                // Check context - look for sequential voltage values
                if i + 16 <= data.len() {
                    let mut cursor = Cursor::new(&data[i..i+16]);
                    let mut vals = Vec::new();
                    for _ in 0..4 {
                        if let Ok(v) = cursor.read_u32::<LittleEndian>() {
                            if (700..=1500).contains(&v) {
                                vals.push(v);
                            }
                        }
                    }
                    if vals.len() >= 2 {
                        report.voltage_tables.push(VoltageTable {
                            offset: i as u64,
                            voltage_type: desc.to_string(),
                            values_mv: vals,
                        });
                    }
                }
            }
        }
    }
    
    println!("    Found {} voltage table candidates", report.voltage_tables.len());
}


fn analyze_memory_timings(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing memory timings...".dimmed());
    
    // LPDDR5 timing bytes in SPD: tAA, tRCD, tRP, tRAS
    // Look for timing patterns near SPD signatures
    
    let spd_sig = [0x23u8, 0x11, 0x13, 0x0E];
    let mut i = 0;
    while i < data.len() - 64 {
        if &data[i..i+4] == spd_sig {
            // Found SPD, extract timings
            if i + 0x20 <= data.len() {
                let timing = MemoryTiming {
                    offset: i as u64,
                    tcl: data[i + 0x18],  // tAAmin
                    trcd: data[i + 0x1A], // tRCDmin
                    trp: data[i + 0x1B],  // tRPab
                    tras: data[i + 0x1C], // tRPpb (using as tRAS proxy)
                };
                report.memory_timings.push(timing);
            }
        }
        i += 1;
    }
    
    println!("    Found {} memory timing structures", report.memory_timings.len());
}

fn analyze_fan_curves(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing fan curves...".dimmed());
    
    // Fan curves typically: temp1, speed1, temp2, speed2, ...
    // Temps: 40-95°C, Speeds: 0-100% or 0-255
    
    // Look for ascending temperature sequences
    for i in 0..data.len().saturating_sub(16) {
        let temps: Vec<u8> = (0..8).map(|j| data[i + j * 2]).collect();
        let speeds: Vec<u8> = (0..8).map(|j| data[i + j * 2 + 1]).collect();
        
        // Check if temps are ascending and in valid range
        let valid_temps = temps.windows(2).all(|w| w[0] < w[1]) &&
                         temps.iter().all(|&t| (30..=100).contains(&t));
        let valid_speeds = speeds.iter().all(|&s| s <= 100 || s == 255);
        
        if valid_temps && valid_speeds && temps[0] >= 30 && temps[7] <= 100 {
            report.fan_curves.push(FanCurve {
                offset: i as u64,
                temp_points: temps,
                speed_points: speeds,
            });
        }
    }
    
    println!("    Found {} potential fan curves", report.fan_curves.len());
}

fn analyze_display(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing display info...".dimmed());
    
    // Look for display-related strings
    let display_patterns = [
        (b"eDP".as_slice(), "eDP Panel"),
        (b"1280x800".as_slice(), "LCD Resolution"),
        (b"1280x720".as_slice(), "720p Mode"),
        (b"1920x1080".as_slice(), "1080p Mode"),
        (b"60Hz".as_slice(), "60Hz Refresh"),
        (b"90Hz".as_slice(), "90Hz Refresh"),
    ];
    
    for (pattern, desc) in display_patterns {
        for (i, window) in data.windows(pattern.len()).enumerate() {
            if window == pattern {
                report.display_info.push(DisplayInfo {
                    offset: i as u64,
                    panel_type: desc.to_string(),
                    resolution: String::from_utf8_lossy(pattern).to_string(),
                });
            }
        }
    }
    
    println!("    Found {} display info entries", report.display_info.len());
}

fn analyze_acpi(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing ACPI tables...".dimmed());
    
    // ACPI table signatures (4 bytes)
    let acpi_sigs = [
        b"DSDT", b"SSDT", b"FACP", b"APIC", b"MCFG", 
        b"HPET", b"BGRT", b"FPDT", b"WSMT", b"TPM2",
    ];
    
    for sig in acpi_sigs {
        for (i, window) in data.windows(4).enumerate() {
            if window == sig {
                // Read table length (at offset +4)
                if i + 8 <= data.len() {
                    let mut cursor = Cursor::new(&data[i+4..i+8]);
                    let size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                    if size > 0 && size < 0x100000 {
                        report.acpi_tables.push(AcpiTable {
                            offset: i as u64,
                            signature: String::from_utf8_lossy(sig).to_string(),
                            size,
                        });
                    }
                }
            }
        }
    }
    
    println!("    Found {} ACPI tables", report.acpi_tables.len());
}

fn analyze_boot_config(data: &[u8], report: &mut DeepAnalysisReport) {
    println!("{}", "  Analyzing boot configuration...".dimmed());
    
    // Look for boot-related strings
    let boot_patterns: &[&[u8]] = &[
        b"Boot0000",
        b"Boot0001", 
        b"BootOrdr",
        b"SecureBt",
        b"SetupMod",
    ];
    
    for pattern in boot_patterns {
        // Search as UTF-16LE
        let utf16: Vec<u8> = pattern.iter()
            .flat_map(|&b| [b, 0])
            .collect();
        
        for (i, window) in data.windows(utf16.len()).enumerate() {
            if window == utf16.as_slice() {
                report.boot_entries.push(BootEntry {
                    offset: i as u64,
                    description: String::from_utf8_lossy(pattern).to_string(),
                });
            }
        }
    }
    
    println!("    Found {} boot config entries", report.boot_entries.len());
}

impl DeepAnalysisReport {
    pub fn print(&self) {
        use colored::Colorize;
        
        if !self.gpu_clocks.is_empty() {
            println!("\n{}", "  GPU CLOCKS:".bold());
            for entry in self.gpu_clocks.iter().take(10) {
                println!("    @ 0x{:08X}: {}-{} MHz (default: {})", 
                    entry.offset, entry.min_mhz, entry.max_mhz, entry.default_mhz);
            }
        }
        
        if !self.voltage_tables.is_empty() {
            println!("\n{}", "  VOLTAGE TABLES:".bold());
            for entry in self.voltage_tables.iter().take(10) {
                println!("    @ 0x{:08X}: {} - {:?}mV", 
                    entry.offset, entry.voltage_type, entry.values_mv);
            }
        }
        
        if !self.memory_timings.is_empty() {
            println!("\n{}", "  MEMORY TIMINGS:".bold());
            for entry in &self.memory_timings {
                println!("    @ 0x{:08X}: tCL=0x{:02X} tRCD=0x{:02X} tRP=0x{:02X} tRAS=0x{:02X}", 
                    entry.offset, entry.tcl, entry.trcd, entry.trp, entry.tras);
            }
        }
        
        if !self.acpi_tables.is_empty() {
            println!("\n{}", "  ACPI TABLES:".bold());
            for entry in &self.acpi_tables {
                println!("    @ 0x{:08X}: {} (size: 0x{:X})", 
                    entry.offset, entry.signature, entry.size);
            }
        }
    }
}
