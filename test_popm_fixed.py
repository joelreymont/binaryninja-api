#!/usr/bin/env python3
"""
Test POPM LLIL with the fixed plugin
"""

import sys
import os
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn

print("=" * 70)
print("Testing POPM LLIL after RRC PC fix")
print("=" * 70)

# Remove any database files to force fresh analysis
test_binary = "/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf"
bndb_file = test_binary + ".bndb"
if os.path.exists(bndb_file):
    os.remove(bndb_file)
    print(f"Removed old database: {bndb_file}")

# Load with fresh analysis
print(f"\nLoading: {test_binary}")
bv = bn.load(test_binary)
if not bv:
    print("Failed to load binary")
    sys.exit(1)

# Wait for analysis
print("Analyzing...")
bv.update_analysis_and_wait()
print("Analysis complete\n")

# Get the use_pushm_popm function
funcs = bv.get_functions_containing(0x4424)
if not funcs:
    print("Function not found")
    sys.exit(1)

func = funcs[0]
print(f"Function: {func.name} @ {hex(func.start)}")

# Check LLIL
llil = func.low_level_il
if not llil:
    print("No LLIL available")
    sys.exit(1)

print(f"\nLLIL Instructions:")
idx = 0
for instr in llil.instructions:
    print(f"  {idx:3d} @ {hex(instr.address):6s}: {instr}")
    idx += 1

# Check if we have LLIL at 0x442e
print(f"\nLooking for LLIL at 0x442e...")
found = False
for instr in llil.instructions:
    if instr.address == 0x442e:
        print(f"  FOUND: {instr}")
        found = True

if not found:
    print("  NOT FOUND - POPM LLIL still missing")
else:
    print("  SUCCESS - POPM LLIL is now generated!")

# Also check the instruction at 0x4428 to see if RRC PC is now unimplemented
print(f"\nChecking RRC PC instruction at 0x4428:")
for instr in llil.instructions:
    if instr.address == 0x4428:
        print(f"  {instr}")
        if "unimplemented" in str(instr).lower():
            print("  ✓ RRC PC is now marked as unimplemented")
        elif "pc =" in str(instr).lower():
            print("  ✗ Still setting PC (old code)")

print("\n" + "=" * 70)
