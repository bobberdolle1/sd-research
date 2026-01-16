//! Pattern definitions for BIOS analysis

/// UEFI Volume signatures
pub const EFI_FV_SIGNATURE: &[u8] = b"_FVH";
pub const EFI_FFS_SIGNATURE: &[u8] = &[0xAA, 0x55];

/// SPD signature for AMD/Valve LPDDR5
pub const SPD_SIGNATURE: &[u8] = &[0x23, 0x11, 0x13, 0x0E];

/// Frequency table patterns
pub const FREQ_PATTERN_51: &[u8] = &[0x51, 0x00, 0x52, 0x00, 0x53, 0x00];
pub const FREQ_PATTERN_59: &[u8] = &[0x59, 0x00, 0x5A, 0x00, 0x5B, 0x00];

/// Power limit values (in mW, little-endian u32)
pub const POWER_15W: u32 = 15000;
pub const POWER_25W: u32 = 25000;
pub const POWER_30W: u32 = 30000;

/// Known UEFI GUIDs
pub struct KnownGuid {
    pub bytes: [u8; 16],
    pub name: &'static str,
}

pub const KNOWN_GUIDS: &[KnownGuid] = &[
    KnownGuid {
        bytes: [0x7A, 0x9A, 0x76, 0x04, 0x42, 0x78, 0x4C, 0x2D, 
                0xA0, 0x17, 0x52, 0x65, 0x4E, 0x74, 0x63, 0x68],
        name: "AmdCbsSetupDxe",
    },
    KnownGuid {
        bytes: [0xC5, 0xB9, 0xD9, 0x3B, 0x7A, 0x5E, 0x4B, 0x99,
                0x8B, 0x47, 0x8E, 0x05, 0x77, 0xD1, 0xE2, 0x5E],
        name: "AmdPbsSetupDxe",
    },
];

/// SMU message patterns
pub const SMU_MSG_PATTERN: &[u8] = b"SMU msg";
pub const SMU_FW_PATTERN: &[u8] = b"SMU FW";

/// PSP signatures
pub const PSP_SIGNATURE: &[u8] = &[0x24, 0x50, 0x53, 0x50]; // $PSP

/// EC patterns
pub const EC_ITE_PATTERN: &[u8] = b"ITE";
