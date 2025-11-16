#!/usr/bin/env python3
"""
Read raw bytes at specific addresses
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

bv = bn.load("/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf")

# Read bytes at use_mova (0x440a)
print("use_mova @ 0x440a:")
data = bv.read(0x440a, 10)
print(f"  Bytes: {data.hex()}")
for i in range(0, len(data), 2):
    word = int.from_bytes(data[i:i+2], 'little')
    print(f"  {hex(0x440a + i)}: {word:04x}")

print()

# Read bytes at use_calla (0x441c)
print("use_calla @ 0x441c:")
data = bv.read(0x441c, 10)
print(f"  Bytes: {data.hex()}")
for i in range(0, len(data), 2):
    word = int.from_bytes(data[i:i+2], 'little')
    print(f"  {hex(0x441c + i)}: {word:04x}")
