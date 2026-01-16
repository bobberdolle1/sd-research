#!/usr/bin/env python3
"""
Steam Deck Memory Overclock
===========================
Патч BIOS для разгона памяти LPDDR5 до ~7000 MT/s.

Поддерживает:
- Сырые дампы (131stockk.bin)
- Подписанные BIOS (F7A0131_sign.fd, F7A0133_sign.fd)

Использование:
  sudo python3 patcher.py                    # автопоиск
  sudo python3 patcher.py F7A0133_sign.fd    # конкретный файл

После прошивки настрой CBS меню!
"""

import os
import sys
import subprocess
import shutil

UPDATER = "/usr/share/jupiter_bios_updater/h2offt"

# Паттерны
FREQ_PATTERN = bytes([0x59, 0x00, 0x5A, 0x00, 0x5B, 0x00])
FREQ_NEW = 0x5F

SPD_SIG = bytes([0x23, 0x11, 0x13, 0x0E])
TCK_LOCKED = 0x0A
TCK_UNLOCK = 0x02
OFF_TCK = 0x0C


def find_bios():
    """Ищет BIOS файл."""
    # Приоритет: подписанные > сырые
    candidates = []
    
    # Подписанные в системе
    if os.path.exists("/usr/share/jupiter_bios/"):
        for f in os.listdir("/usr/share/jupiter_bios/"):
            if f.endswith("_sign.fd"):
                candidates.append(("/usr/share/jupiter_bios/" + f, True))
    
    # Локальные файлы
    for f in os.listdir("."):
        if f.endswith("_sign.fd"):
            candidates.append((f, True))
        elif f.endswith(".bin") and "stock" in f.lower():
            candidates.append((f, False))
    
    # Сортируем по версии (новые первые)
    candidates.sort(key=lambda x: x[0], reverse=True)
    
    return candidates[0] if candidates else (None, False)


def patch_bios(data):
    """Патчит BIOS данные."""
    
    # Патч 1: Таблица частот
    print("\n[1/2] Патч таблицы частот...")
    freq_patched = 0
    off = 0
    while True:
        off = data.find(FREQ_PATTERN, off)
        if off == -1:
            break
        print(f"    [PATCH] 0x{off:08X}: 0x59 -> 0x{FREQ_NEW:02X}")
        data[off] = FREQ_NEW
        freq_patched += 1
        off += 1
    
    if freq_patched == 0:
        print("    [!] Таблица частот не найдена!")
    else:
        print(f"    [+] Пропатчено: {freq_patched}")
    
    # Патч 2: SPD unlock
    print("\n[2/2] SPD unlock...")
    spd_unlocked = 0
    off = 0
    while True:
        off = data.find(SPD_SIG, off)
        if off == -1 or off + 0x20 > len(data):
            break
        
        tck = data[off + OFF_TCK]
        if tck == TCK_LOCKED:
            data[off + OFF_TCK] = TCK_UNLOCK
            spd_unlocked += 1
            vendor = data[off+4:off+8].hex().upper()
            print(f"    [UNLOCK] 0x{off:08X} vendor={vendor}")
        
        off += 1
    
    print(f"    [+] Разблокировано: {spd_unlocked}")
    
    return freq_patched > 0 or spd_unlocked > 0


def flash_bios(bios_file):
    """Прошивает BIOS."""
    if not os.path.exists(UPDATER):
        print(f"[!] {UPDATER} не найден")
        return False
    
    print("\n" + "=" * 50)
    print("       !!! НЕ ВЫКЛЮЧАЙ КОНСОЛЬ !!!")
    print("=" * 50)
    
    cmd = [UPDATER, bios_file, "-all"]
    print(f"[*] {' '.join(cmd)}")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    print(result.stdout)
    if result.stderr:
        print(result.stderr)
    
    if result.returncode == 0:
        print("\n[+] ПРОШИВКА УСПЕШНА!")
        return True
    
    print("\n[!] Ошибка прошивки")
    return False


def main():
    print("""
╔═══════════════════════════════════════════════════╗
║   STEAM DECK MEMORY OVERCLOCK -> ~7000 MT/s       ║
║                                                   ║
║   Freq: 0x59 -> 0x5F (3200MHz = ~7000 MT/s)       ║
║   SPD:  tCK unlock                                ║
╚═══════════════════════════════════════════════════╝
""")

    # Определяем входной файл
    if len(sys.argv) > 1:
        input_file = sys.argv[1]
        is_signed = input_file.endswith("_sign.fd")
    else:
        input_file, is_signed = find_bios()
        if not input_file:
            print("[!] BIOS файл не найден!")
            print("[*] Использование: python3 patcher.py <bios.fd>")
            return
    
    if not os.path.exists(input_file):
        print(f"[!] Файл не найден: {input_file}")
        return
    
    # Читаем
    print(f"[*] Файл: {input_file}")
    with open(input_file, "rb") as f:
        data = bytearray(f.read())
    print(f"[*] Размер: {len(data)} bytes ({len(data)//1024//1024} MB)")
    print(f"[*] Тип: {'подписанный' if is_signed else 'сырой дамп'}")
    
    # Патчим
    if not patch_bios(data):
        print("[!] Патч не применился!")
        return
    
    # Сохраняем
    output_file = "bios_patched.fd" if is_signed else "bios_modded.bin"
    with open(output_file, "wb") as f:
        f.write(data)
    print(f"\n[+] Сохранено: {output_file}")
    
    # Прошиваем
    if os.path.exists(UPDATER):
        answer = input("\nПрошить? (yes/no): ").strip().lower()
        if answer in ['yes', 'y', 'да', 'flash']:
            if flash_bios(output_file):
                print("""
╔═══════════════════════════════════════════════════╗
║  ПОСЛЕ ПЕРЕЗАГРУЗКИ:                              ║
║                                                   ║
║  1. CMOS reset: Vol(-) + (...) + Power 10с        ║
║  2. BIOS: Vol(+) + Power                          ║
║  3. Boot From File -> RUNTIME-PATCHER.efi         ║
║  4. CBS -> Overclock: Enabled                     ║
║  5. Memory Clock: 3200MHz (= ~7000 MT/s)          ║
║  6. Save                                          ║
║                                                   ║
║  Восстановление: Vol(-)+(...)+Power 15с           ║
╚═══════════════════════════════════════════════════╝
""")
    else:
        print(f"\n[*] Скопируй {output_file} на Steam Deck и прошей")


if __name__ == "__main__":
    main()
