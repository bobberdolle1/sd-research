#!/usr/bin/env python3
"""Deep scan BIOS for interesting structures"""
import re
import struct

with open('F7A0133_sign.fd', 'rb') as f:
    data = f.read()

print('=== AGESA VERSION ===')
for m in re.finditer(rb'AGESA[^\x00]{0,50}', data):
    s = m.group().decode('ascii', errors='ignore')
    print(f'  @ 0x{m.start():08X}: {s}')

print('\n=== SMU FIRMWARE INFO ===')
for m in re.finditer(rb'SMU[^\x00]{0,30}', data):
    print(f'  @ 0x{m.start():08X}: {m.group()[:40]}')

print('\n=== POWER TABLE CANDIDATES ===')
pw_15 = struct.pack('<I', 15000)
for m in re.finditer(re.escape(pw_15), data):
    ctx = data[m.start()-32:m.start()+64]
    if len(ctx) >= 96:
        vals = struct.unpack('<24I', ctx)
        power_vals = [v for v in vals if 1000 <= v <= 50000 and v % 1000 == 0]
        if len(power_vals) >= 2:
            print(f'  @ 0x{m.start():08X}: {power_vals}')

print('\n=== ALL SPD STRUCTURES DETAILED ===')
spd_sig = bytes([0x23, 0x11, 0x13, 0x0E])
for m in re.finditer(re.escape(spd_sig), data):
    offset = m.start()
    spd = data[offset:offset+64]
    vendor = spd[4:8].hex() if len(spd) > 8 else 'N/A'
    tck = spd[0x0C] if len(spd) > 0x0C else 0
    # Decode more fields
    print(f'\n  SPD @ 0x{offset:08X}:')
    print(f'    Vendor ID: {vendor}')
    print(f'    tCK: 0x{tck:02X} ({"LOCKED" if tck == 0x0A else "UNLOCKED" if tck == 0x02 else "OTHER"})')
    print(f'    Raw: {spd[:32].hex()}')

print('\n=== FREQUENCY TABLE ANALYSIS ===')
# Наша известная таблица
for pattern_name, pattern in [
    ('0x51 sequence', bytes([0x51, 0x00, 0x52, 0x00, 0x53, 0x00])),
    ('0x59 sequence', bytes([0x59, 0x00, 0x5A, 0x00, 0x5B, 0x00])),
]:
    for m in re.finditer(re.escape(pattern), data):
        print(f'\n  {pattern_name} @ 0x{m.start():08X}:')
        chunk = data[m.start():m.start()+48]
        # Decode as uint16
        vals = [struct.unpack('<H', chunk[i:i+2])[0] for i in range(0, len(chunk), 2)]
        print(f'    Values: {[hex(v) for v in vals if v != 0xFFFF]}')

print('\n=== LOOKING FOR STAPM/PPT STRINGS ===')
for kw in [b'STAPM', b'PPT', b'TDP', b'Sustained', b'Peak', b'Fast', b'Slow']:
    for m in re.finditer(kw, data):
        ctx = data[max(0,m.start()-16):m.start()+48]
        print(f'  {kw.decode()} @ 0x{m.start():08X}')

print('\n=== UEFI VARIABLE CANDIDATES (Memory Clock) ===')
# Ищем GUID-подобные структуры рядом с memory-related данными
# Memory Clock variable offset 0x31F из документации
mem_clock_patterns = [
    b'\x1F\x03',  # 0x31F little-endian
    b'\x31\x0F',  # альтернатива
]
for p in mem_clock_patterns:
    matches = list(re.finditer(re.escape(p), data))
    print(f'  Pattern {p.hex()}: {len(matches)} matches')
