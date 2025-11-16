#!/usr/bin/env python3
"""
Scan binary for MSP430X extension words and MOVA/CALLA patterns
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

bv = bn.load("/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf")

# Scan code section for extension words (0x1800-0x1FFF)
print("Scanning for extension words (0x1800-0x1FFF):")
print("=" * 70)

code_start = 0x4400
code_end = 0x4500

found_any = False
for addr in range(code_start, code_end, 2):
    data = bv.read(addr, 2)
    if len(data) == 2:
        word = int.from_bytes(data, 'little')
        if (word & 0xF800) == 0x1800:
            print(f"  {hex(addr)}: {word:04x} - Extension word found!")
            # Read next word to see instruction
            next_data = bv.read(addr + 2, 2)
            if len(next_data) == 2:
                next_word = int.from_bytes(next_data, 'little')
                print(f"    Next word: {next_word:04x}")
            found_any = True

if not found_any:
    print("  No extension words found in code section")

# Check all functions for their content
print("\n" + "=" * 70)
print("All functions in binary:")
print("=" * 70)

for func in bv.functions:
    print(f"\n{func.name} @ {hex(func.start)}:")
    # Show first few instructions
    count = 0
    for bb in func.basic_blocks:
        for line in bb.disassembly_text:
            if count < 3:
                addr = line.address
                tokens_text = ''.join(str(t) for t in line.tokens)
                print(f"  {hex(addr)}: {tokens_text}")
                count += 1
        if count >= 3:
            break
