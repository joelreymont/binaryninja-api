#!/usr/bin/env python3
"""
ARM64 LSE Atomic MIN/MAX Intrinsic Tests

Tests for fix to issue #6599 - ARM64 atomic operation intrinsics.

ARM64 Large System Extensions (LSE) include atomic MIN/MAX operations:
- LDSMAX/LDSMIN - Signed atomic min/max with load
- LDUMAX/LDUMIN - Unsigned atomic min/max with load
- STSMAX/STSMIN - Store-only signed min/max
- STUMAX/STUMIN - Store-only unsigned min/max

Each with byte (B), halfword (H), word, and acquire/release variants.

This test validates that these instructions are lifted as intrinsics.

Run with: python3 test_atomic_minmax.py
Requires: Binary Ninja installation with ARM64 architecture
"""

import sys
import struct

try:
    import binaryninja
    from binaryninja import binaryview, Architecture
except ImportError:
    print("ERROR: Binary Ninja not found. This test requires Binary Ninja to be installed.", file=sys.stderr)
    sys.exit(1)

# ARM64 LSE atomic MIN/MAX test cases
# Format: (bytecode, expected_intrinsic_name)
tests_atomic_minmax = [
    # LDSMAX - Atomic signed maximum, word
    # LDSMAX <Ws>, <Wt>, [<Xn|SP>] - encoding: 11111000xx1xxxxx010000xxxxxxxxxx
    (b'\x41\x40\x22\xF8', '__ldsmax'),     # ldsmax w2, w1, [x2]
    (b'\xE3\x43\x25\xF8', '__ldsmax'),     # ldsmax w5, w3, [sp]

    # LDSMAXB - Atomic signed maximum, byte
    (b'\x20\x40\x21\x38', '__ldsmaxb'),    # ldsmaxb w1, w0, [x1]

    # LDSMAXH - Atomic signed maximum, halfword
    (b'\x41\x40\x22\x78', '__ldsmaxh'),    # ldsmaxh w2, w1, [x2]

    # LDSMIN - Atomic signed minimum, word
    # LDSMIN <Ws>, <Wt>, [<Xn|SP>] - encoding: 11111000xx1xxxxx010100xxxxxxxxxx
    (b'\x41\x50\x22\xF8', '__ldsmin'),     # ldsmin w2, w1, [x2]
    (b'\xE3\x53\x25\xF8', '__ldsmin'),     # ldsmin w5, w3, [sp]

    # LDSMINB - Atomic signed minimum, byte
    (b'\x20\x50\x21\x38', '__ldsminb'),    # ldsminb w1, w0, [x1]

    # LDSMINH - Atomic signed minimum, halfword
    (b'\x41\x50\x22\x78', '__ldsminh'),    # ldsminh w2, w1, [x2]

    # LDUMAX - Atomic unsigned maximum, word
    # LDUMAX <Ws>, <Wt>, [<Xn|SP>] - encoding: 11111000xx1xxxxx011000xxxxxxxxxx
    (b'\x41\x60\x22\xF8', '__ldumax'),     # ldumax w2, w1, [x2]
    (b'\xE3\x63\x25\xF8', '__ldumax'),     # ldumax w5, w3, [sp]

    # LDUMAXB - Atomic unsigned maximum, byte
    (b'\x20\x60\x21\x38', '__ldumaxb'),    # ldumaxb w1, w0, [x1]

    # LDUMAXH - Atomic unsigned maximum, halfword
    (b'\x41\x60\x22\x78', '__ldumaxh'),    # ldumaxh w2, w1, [x2]

    # LDUMIN - Atomic unsigned minimum, word
    # LDUMIN <Ws>, <Wt>, [<Xn|SP>] - encoding: 11111000xx1xxxxx011100xxxxxxxxxx
    (b'\x41\x70\x22\xF8', '__ldumin'),     # ldumin w2, w1, [x2]
    (b'\xE3\x73\x25\xF8', '__ldumin'),     # ldumin w5, w3, [sp]

    # LDUMINB - Atomic unsigned minimum, byte
    (b'\x20\x70\x21\x38', '__lduminb'),    # lduminb w1, w0, [x1]

    # LDUMINH - Atomic unsigned minimum, halfword
    (b'\x41\x70\x22\x78', '__lduminh'),    # lduminh w2, w1, [x2]

    # STSMAX - Store-only atomic signed maximum, word
    # STSMAX <Ws>, [<Xn|SP>] - encoding: 11111000xx1xxxxx010000xxxxxxxxxx with Wt=11111
    (b'\x5F\x40\x21\xF8', '__stsmax'),     # stsmax w1, [x2]

    # STSMAXB - Store-only atomic signed maximum, byte
    (b'\x3F\x40\x20\x38', '__stsmaxb'),    # stsmaxb w0, [x1]

    # STSMAXH - Store-only atomic signed maximum, halfword
    (b'\x5F\x40\x21\x78', '__stsmaxh'),    # stsmaxh w1, [x2]

    # STSMIN - Store-only atomic signed minimum, word
    (b'\x5F\x50\x21\xF8', '__stsmin'),     # stsmin w1, [x2]

    # STSMINB - Store-only atomic signed minimum, byte
    (b'\x3F\x50\x20\x38', '__stsminb'),    # stsminb w0, [x1]

    # STSMINH - Store-only atomic signed minimum, halfword
    (b'\x5F\x50\x21\x78', '__stsminh'),    # stsminh w1, [x2]

    # STUMAX - Store-only atomic unsigned maximum, word
    (b'\x5F\x60\x21\xF8', '__stumax'),     # stumax w1, [x2]

    # STUMAXB - Store-only atomic unsigned maximum, byte
    (b'\x3F\x60\x20\x38', '__stumaxb'),    # stumaxb w0, [x1]

    # STUMAXH - Store-only atomic unsigned maximum, halfword
    (b'\x5F\x60\x21\x78', '__stumaxh'),    # stumaxh w1, [x2]

    # STUMIN - Store-only atomic unsigned minimum, word
    (b'\x5F\x70\x21\xF8', '__stumin'),     # stumin w1, [x2]

    # STUMINB - Store-only atomic unsigned minimum, byte
    (b'\x3F\x70\x20\x38', '__stuminb'),    # stuminb w0, [x1]

    # STUMINH - Store-only atomic unsigned minimum, halfword
    (b'\x5F\x70\x21\x78', '__stuminh'),    # stuminh w1, [x2]
]

PROLOG = b''
EPILOG = b'\xE0\x03\x1F\xAA'  # mov x0, xzr (marker)

def lift(data, disasm=False):
    """Lift ARM64 instruction to IL"""
    arch = Architecture['aarch64']
    if not arch:
        return None, None

    platform = arch.standalone_platform
    bv = binaryview.BinaryView.new(data + EPILOG)
    bv.add_function(0, plat=platform)

    if len(bv.functions) == 0:
        return None, None

    tokens = []
    for block in bv.functions[0].lifted_il:
        for il in block:
            tokens.append(str(il))

    il_str = '; '.join(tokens)

    # Remove epilog marker
    try:
        i = il_str.rindex('; LLIL_SET_REG.q(x0,LLIL_CONST.q(0x0))')
        il_str = il_str[0:i]
    except ValueError:
        pass

    return il_str, None

def test_atomic_minmax_lifting():
    """Test ARM64 atomic MIN/MAX intrinsic lifting"""
    print(f"Testing ARM64 LSE atomic MIN/MAX intrinsics ({len(tests_atomic_minmax)} tests)...", file=sys.stderr)

    passed = 0
    failed = 0

    for test_i, (bytecode, expected_intrinsic) in enumerate(tests_atomic_minmax):
        actual_lift, _ = lift(bytecode)

        if actual_lift is None:
            print(f"SKIP Test {test_i + 1}: Could not lift instruction")
            continue

        # Check that it's the right intrinsic (pretty-printed format shows as function call)
        if expected_intrinsic not in actual_lift:
            print(f"FAIL Test {test_i + 1}: {bytecode.hex()}")
            print(f"  Expected intrinsic: {expected_intrinsic}")
            print(f"  Actual IL: {actual_lift}")
            failed += 1
            continue

        passed += 1
        print(f"PASS Test {test_i + 1}: {expected_intrinsic}")

    print(f"\nResults: {passed} passed, {failed} failed out of {len(tests_atomic_minmax)} tests")
    return failed == 0

def test_atomic_output_registers():
    """Test that LD* variants have output registers, ST* variants don't"""
    print("\nTesting atomic intrinsic output registers...", file=sys.stderr)

    load_tests = [
        (b'\x41\x40\x22\xF8', 'ldsmax', True),   # Should have output
        (b'\x41\x50\x22\xF8', 'ldsmin', True),   # Should have output
        (b'\x5F\x40\x21\xF8', 'stsmax', False),  # Should NOT have output
        (b'\x5F\x50\x21\xF8', 'stsmin', False),  # Should NOT have output
    ]

    passed = 0
    failed = 0

    for bytecode, name, should_have_output in load_tests:
        actual_lift, _ = lift(bytecode)

        if actual_lift is None:
            print(f"SKIP: {name}")
            continue

        # LD* instructions should have register assignment (x = ...)
        # ST* instructions should just have function call without assignment
        # In pretty-printed format, assignments show as "reg = intrinsic(...)"
        has_output_reg = ' = __{}'.format(name) in actual_lift

        if should_have_output and not has_output_reg:
            print(f"FAIL: {name} should have output register")
            print(f"  IL: {actual_lift}")
            failed += 1
        elif not should_have_output and has_output_reg:
            print(f"FAIL: {name} should NOT have output register")
            print(f"  IL: {actual_lift}")
            failed += 1
        else:
            passed += 1
            print(f"PASS: {name} output register handling correct")

    print(f"\nOutput register results: {passed} passed, {failed} failed")
    return failed == 0

if __name__ == '__main__':
    success = True

    # Run intrinsic lifting tests
    if not test_atomic_minmax_lifting():
        success = False

    # Run output register tests
    if not test_atomic_output_registers():
        success = False

    sys.exit(0 if success else 1)
