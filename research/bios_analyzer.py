#!/usr/bin/env python3
"""
BIOS F7A0133 Deep Analyzer
Ищем интересные функции для разгона и оптимизации Steam Deck
"""

import re
import struct
from collections import defaultdict

BIOS_FILE = "F7A0133_sign.fd"

# Паттерны для поиска (ASCII и UTF-16LE)
SEARCH_PATTERNS = [
    # Power Management
    b"Power Limit", b"TDP", b"PPT", b"STAPM", b"cTDP",
    b"Fast PPT", b"Slow PPT", b"Sustained Power",
    b"Peak Power", b"Skin Temp", b"APU Power",
    
    # CPU/GPU
    b"CPU Boost", b"Max Frequency", b"Core Voltage",
    b"GPU Clock", b"GFX Voltage", b"VDDCR", b"SoC Voltage",
    b"Precision Boost", b"PBO", b"Curve Optimizer",
    
    # Memory
    b"Memory Clock", b"VDDQ", b"VDD2", b"VDDIO",
    b"Infinity Fabric", b"FCLK", b"UCLK", b"MCLK",
    b"Memory Training", b"tCL", b"tRCD", b"tRP", b"tRAS",
    
    # SMU / Firmware
    b"SMU", b"AGESA", b"PSP", b"Overclock", b"OC Mode",
    
    # Fan / Thermal
    b"Fan", b"Thermal", b"Temperature", b"Throttle",
    
    # Display
    b"Display", b"Refresh Rate", b"Panel", b"eDP",
    
    # Steam Deck specific
    b"Jupiter", b"Aerith", b"Valve", b"Steam",
]

def search_strings(data, min_len=6):
    """Извлекаем все ASCII строки"""
    strings = []
    # ASCII
    for match in re.finditer(rb'[\x20-\x7e]{%d,}' % min_len, data):
        strings.append((match.start(), match.group().decode('ascii', errors='ignore')))
    return strings

def search_utf16_strings(data, min_len=4):
    """Извлекаем UTF-16LE строки (UEFI часто использует)"""
    strings = []
    # UTF-16LE pattern: printable char + \x00
    pattern = rb'(?:[\x20-\x7e]\x00){%d,}' % min_len
    for match in re.finditer(pattern, data):
        try:
            s = match.group().decode('utf-16-le')
            if len(s) >= min_len:
                strings.append((match.start(), s))
        except:
            pass
    return strings

def find_pattern_context(data, pattern, context_bytes=64):
    """Находим паттерн и показываем контекст вокруг"""
    results = []
    # ASCII
    for match in re.finditer(re.escape(pattern), data):
        start = max(0, match.start() - context_bytes)
        end = min(len(data), match.end() + context_bytes)
        results.append((match.start(), data[start:end]))
    # UTF-16LE
    utf16_pattern = pattern.decode('ascii', errors='ignore').encode('utf-16-le')
    for match in re.finditer(re.escape(utf16_pattern), data):
        start = max(0, match.start() - context_bytes)
        end = min(len(data), match.end() + context_bytes)
        results.append((match.start(), data[start:end]))
    return results

def find_numeric_tables(data):
    """Ищем таблицы с числовыми значениями (частоты, вольтажи)"""
    tables = []
    
    # Паттерн частот памяти (как в memory-clock.txt): последовательные значения 0x51-0x5F
    freq_pattern = bytes([0x51, 0x00, 0x52, 0x00, 0x53, 0x00])
    for match in re.finditer(re.escape(freq_pattern), data):
        tables.append(("FREQ_TABLE_51", match.start(), data[match.start():match.start()+32]))
    
    # Паттерн 0x59 0x00 0x5A 0x00 (наш известный)
    freq_pattern2 = bytes([0x59, 0x00, 0x5A, 0x00, 0x5B, 0x00])
    for match in re.finditer(re.escape(freq_pattern2), data):
        tables.append(("FREQ_TABLE_59", match.start(), data[match.start():match.start()+32]))
    
    # Power limits (типичные значения 15W=0x0F, 25W=0x19, 30W=0x1E в разных форматах)
    # 15000 mW = 0x3A98, 25000 mW = 0x61A8
    for val, name in [(0x3A98, "15W"), (0x61A8, "25W"), (0x7530, "30W")]:
        pattern = struct.pack('<H', val)
        for match in re.finditer(re.escape(pattern), data):
            context = data[max(0,match.start()-16):match.start()+16]
            tables.append((f"POWER_{name}", match.start(), context))
    
    return tables

def find_spd_structures(data):
    """Ищем SPD структуры памяти"""
    spd_sig = bytes([0x23, 0x11, 0x13, 0x0E])  # AMD/Valve LPDDR5 SPD
    results = []
    for match in re.finditer(re.escape(spd_sig), data):
        offset = match.start()
        spd_data = data[offset:offset+64]
        tck = spd_data[0x0C] if len(spd_data) > 0x0C else 0
        results.append((offset, tck, spd_data[:32].hex()))
    return results

def find_guid_modules(data):
    """Ищем UEFI модули по GUID"""
    # Известные интересные GUID
    known_guids = {
        "AmdCbsSetupDxe": bytes.fromhex("8A96C9E8D4E94F1E9B5E3A3B8C9D0E1F"),  # примерный
        "PlatformSetup": bytes.fromhex("A04A27F456AE4DA18E7B456789ABCDEF"),
    }
    # Ищем паттерн GUID (16 байт с характерной структурой)
    results = []
    # Типичный UEFI FFS header
    for match in re.finditer(rb'\x00{0,4}[\x00-\xff]{16}\x00{0,4}', data):
        pass  # слишком много false positives
    return results

def analyze_menu_structures(data):
    """Ищем структуры меню BIOS (IFR-подобные)"""
    results = []
    
    # Ищем строки с "MHz", "mV", "W" рядом с числами
    patterns = [
        rb'\d{3,4}\s*MHz',
        rb'\d{2,4}\s*mV', 
        rb'\d{1,2}\s*W\b',
        rb'0x[0-9A-Fa-f]{2}',
    ]
    
    for p in patterns:
        for match in re.finditer(p, data):
            context_start = max(0, match.start() - 32)
            context = data[context_start:match.end() + 32]
            results.append((match.start(), match.group(), context))
    
    return results[:50]  # лимит

def main():
    print("=" * 70)
    print("BIOS F7A0133 DEEP ANALYZER")
    print("=" * 70)
    
    with open(BIOS_FILE, 'rb') as f:
        data = f.read()
    
    print(f"\n[INFO] File size: {len(data):,} bytes ({len(data)/1024/1024:.2f} MB)")
    
    # 1. Поиск ключевых строк
    print("\n" + "=" * 70)
    print("1. КЛЮЧЕВЫЕ СТРОКИ (Power, Memory, CPU, GPU)")
    print("=" * 70)
    
    found_patterns = {}
    for pattern in SEARCH_PATTERNS:
        results = find_pattern_context(data, pattern, context_bytes=32)
        if results:
            found_patterns[pattern.decode('ascii', errors='ignore')] = results
    
    for name, results in sorted(found_patterns.items()):
        print(f"\n[FOUND] '{name}' - {len(results)} occurrences")
        for offset, context in results[:3]:  # показываем первые 3
            print(f"  @ 0x{offset:08X}")
    
    # 2. SPD структуры
    print("\n" + "=" * 70)
    print("2. SPD СТРУКТУРЫ ПАМЯТИ")
    print("=" * 70)
    
    spd_results = find_spd_structures(data)
    print(f"Found {len(spd_results)} SPD structures (signature 23 11 13 0E)")
    for offset, tck, hex_preview in spd_results:
        tck_status = "LOCKED" if tck == 0x0A else "UNLOCKED" if tck == 0x02 else f"0x{tck:02X}"
        print(f"  @ 0x{offset:08X}: tCK={tck_status}")
        print(f"    {hex_preview}")
    
    # 3. Таблицы частот
    print("\n" + "=" * 70)
    print("3. ТАБЛИЦЫ ЧАСТОТ И POWER LIMITS")
    print("=" * 70)
    
    tables = find_numeric_tables(data)
    for name, offset, context in tables[:20]:
        print(f"  [{name}] @ 0x{offset:08X}: {context.hex()}")
    
    # 4. Интересные строки (UTF-16)
    print("\n" + "=" * 70)
    print("4. ИНТЕРЕСНЫЕ UTF-16 СТРОКИ (меню BIOS)")
    print("=" * 70)
    
    utf16_strings = search_utf16_strings(data, min_len=8)
    
    # Фильтруем интересные
    keywords = ['clock', 'power', 'volt', 'freq', 'memory', 'cpu', 'gpu', 'fan', 
                'thermal', 'boost', 'limit', 'tdp', 'smu', 'pbo', 'overclock',
                'mhz', 'ghz', 'mv', 'speed', 'performance', 'turbo']
    
    interesting = []
    for offset, s in utf16_strings:
        s_lower = s.lower()
        if any(kw in s_lower for kw in keywords):
            interesting.append((offset, s))
    
    for offset, s in interesting[:50]:
        print(f"  @ 0x{offset:08X}: {s}")
    
    # 5. Все уникальные строки с числами (потенциальные настройки)
    print("\n" + "=" * 70)
    print("5. СТРОКИ С ЧИСЛОВЫМИ ЗНАЧЕНИЯМИ")
    print("=" * 70)
    
    menu_items = analyze_menu_structures(data)
    seen = set()
    for offset, match, context in menu_items:
        match_str = match.decode('ascii', errors='ignore')
        if match_str not in seen:
            seen.add(match_str)
            print(f"  @ 0x{offset:08X}: {match_str}")

if __name__ == "__main__":
    main()
