#!/usr/bin/env python3
"""
Inspect LLIL for PUSHM/POPM to see SP modifications
"""

import sys
sys.path.insert(0, '/root/binaryninja/python')

import binaryninja as bn
import os

# Load binary
test_binary = "/home/user/binaryninja-api/test_binaries/msp430x/test_simple.elf"
bndb = test_binary + ".bndb"
if os.path.exists(bndb):
    os.remove(bndb)

bv = bn.load(test_binary)
bv.update_analysis_and_wait()

# Get function containing PUSHM/POPM
func = bv.get_functions_containing(0x4424)[0]
llil = func.low_level_il

print("=" * 70)
print(f"Function: {func.name} @ {hex(func.start)}")
print("=" * 70)

print("\nAll LLIL instructions:")
idx = 0
for instr in llil.instructions:
    # Highlight SP modifications
    marker = ""
    if 'sp' in str(instr).lower() or 'r1' in str(instr).lower():
        marker = " <-- SP MODIFICATION"

    print(f"  {idx:3d} @ {hex(instr.address):6s}: {instr}{marker}")
    idx += 1

print("\n" + "=" * 70)
print("Looking for SP updates...")
print("=" * 70)

sp_updates = []
for instr in llil.instructions:
    instr_str = str(instr).lower()
    if 'sp' in instr_str or 'r1' in instr_str:
        sp_updates.append((instr.address, str(instr)))

if sp_updates:
    print(f"\nFound {len(sp_updates)} SP-related instructions:")
    for addr, instr in sp_updates:
        print(f"  {hex(addr)}: {instr}")
else:
    print("\nNo explicit SP updates found!")
    print("Note: Binary Ninja may handle SP updates implicitly in PUSH/POP")
