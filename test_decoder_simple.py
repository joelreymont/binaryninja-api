#!/usr/bin/env python3
"""
Simple decoder test - just check if instructions decode
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

# Test bytes
tests = [
    (0x4416, b'\x10\x01', 'RETA'),
    (0x4424, b'\x3f\x14', 'PUSHM.A #4, r15'),
    (0x442e, b'\x3c\x16', 'POPM.A #4, r15'),
    (0x4434, b'\x4f\x06', 'RLAM.A #2, r15'),
    (0x4436, b'\x4e\x05', 'RRAM.A #2, r14'),
]

print("MSP430X Decoder Test")
print("=" * 60)

# Get MSP430 architecture
arch = bn.Architecture['msp430']
print(f"Architecture: {arch.name}")
print(f"Max instruction length: {arch.max_instr_length}")
print(f"Address size: {arch.address_size}")
print()

for addr, bytesval, expected in tests:
    print(f"\nTest: {expected} @ {hex(addr)}")
    print(f"Bytes: {bytesval.hex()}")

    # Try to get instruction text
    result = arch.get_instruction_text(bytesval, addr)
    if result:
        tokens, length = result
        text = ''.join(str(t) for t in tokens)
        print(f"Decoded: {text} (length={length})")
    else:
        print("Failed to decode")

    # Try to get instruction info
    info = arch.get_instruction_info(bytesval, addr)
    if info:
        print(f"Info: length={info.length}, branches={len(info.branches)}")
        for branch in info.branches:
            print(f"  Branch: {branch.type} to {hex(branch.target) if branch.target else 'unknown'}")
    else:
        print("No instruction info")

print("\n" + "=" * 60)
