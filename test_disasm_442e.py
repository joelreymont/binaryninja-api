#!/usr/bin/env python3
"""
Check disassembly of the second basic block
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

print("=" * 70)
print("Disassembly at 0x442e basic block")
print("=" * 70)

# Load test binary
bv = bn.load("/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf")
if not bv:
    print("Failed to load binary")
    sys.exit(1)

bv.update_analysis_and_wait()

# Get function
funcs = bv.get_functions_containing(0x4424)
if not funcs:
    print("Function not found")
    sys.exit(1)

func = funcs[0]
arch = bv.arch

print(f"\nFunction: {func.name} @ {hex(func.start)} - {hex(func.highest_address)}")

# Show disassembly of entire function
print(f"\nComplete disassembly:")
for bb in func.basic_blocks:
    print(f"\n  Basic Block: {hex(bb.start)} - {hex(bb.end)}")
    for line in bb.disassembly_text:
        addr = line.address
        tokens_text = ''.join(str(t) for t in line.tokens)

        # Also show the raw bytes
        instr_len = 2  # Most MSP430 instructions are 2 bytes
        data = bv.read(addr, instr_len)

        print(f"    {hex(addr)}: {data.hex():6s} {tokens_text}")

# Check instruction info for each instruction
print(f"\nInstruction info:")
for bb in func.basic_blocks:
    print(f"\n  Basic Block: {hex(bb.start)} - {hex(bb.end)}")
    addr = bb.start
    while addr < bb.end:
        data = bv.read(addr, arch.max_instr_length)
        info = arch.get_instruction_info(data, addr)
        text_result = arch.get_instruction_text(data, addr)

        if info and text_result:
            tokens, length = text_result
            text = ''.join(str(t) for t in tokens)
            print(f"    {hex(addr)}: {text:20s} (len={length}, branches={len(info.branches)})")
            addr += length
        else:
            print(f"    {hex(addr)}: FAILED TO DECODE")
            break

print("\n" + "=" * 70)
