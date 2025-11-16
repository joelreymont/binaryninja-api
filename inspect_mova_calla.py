#!/usr/bin/env python3
"""
Inspect MOVA and CALLA instructions
"""

import sys
import os
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

# Load binary
test_binary = "/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf"
bndb = test_binary + ".bndb"
if os.path.exists(bndb):
    os.remove(bndb)

bv = bn.load(test_binary)
bv.update_analysis_and_wait()

# Check use_mova @ 0x440a
print("=" * 70)
print("use_mova @ 0x440a")
print("=" * 70)

func = bv.get_function_at(0x440a)
if func:
    print(f"\nDisassembly:")
    for bb in func.basic_blocks:
        for line in bb.disassembly_text:
            addr = line.address
            tokens_text = ''.join(str(t) for t in line.tokens)
            data = bv.read(addr, 6)
            print(f"  {hex(addr)}: {data[:4].hex():8s} {tokens_text}")

    llil = func.low_level_il
    if llil:
        print(f"\nLLIL instructions:")
        idx = 0
        for instr in llil.instructions:
            print(f"  {idx:3d} @ {hex(instr.address):6s}: {instr}")
            idx += 1

# Check use_calla @ 0x441c
print("\n" + "=" * 70)
print("use_calla @ 0x441c")
print("=" * 70)

func = bv.get_function_at(0x441c)
if func:
    print(f"\nDisassembly:")
    for bb in func.basic_blocks:
        for line in bb.disassembly_text:
            addr = line.address
            tokens_text = ''.join(str(t) for t in line.tokens)
            data = bv.read(addr, 6)
            print(f"  {hex(addr)}: {data[:4].hex():8s} {tokens_text}")

    llil = func.low_level_il
    if llil:
        print(f"\nLLIL instructions:")
        idx = 0
        for instr in llil.instructions:
            print(f"  {idx:3d} @ {hex(instr.address):6s}: {instr}")
            idx += 1
