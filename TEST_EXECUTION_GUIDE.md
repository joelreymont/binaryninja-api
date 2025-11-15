# Test Execution Guide for Binary Ninja Architecture Fixes

**Date:** 2025-11-15
**For:** Binary Ninja Commercial/Enterprise/Personal License Holders

## Overview

This guide provides step-by-step instructions for validating the architecture fixes using the automated test suite.

## Prerequisites

### Required
- Binary Ninja Commercial, Personal, or Enterprise license
- Binary Ninja installation (tested with version 3.5+)
- Python 3.8 or higher
- Linux, macOS, or Windows

### Binary Ninja Python API Setup

The tests require the Binary Ninja Python API module. To verify it's available:

```bash
python3 -c "import binaryninja; print(f'Binary Ninja {binaryninja.core_version()}')"
```

If this fails, configure your Python environment:

**Linux/macOS:**
```bash
export PYTHONPATH="/path/to/binaryninja/python:$PYTHONPATH"
# Example: export PYTHONPATH="/opt/binaryninja/python:$PYTHONPATH"
```

**Windows:**
```powershell
$env:PYTHONPATH = "C:\Program Files\Vector35\BinaryNinja\python;$env:PYTHONPATH"
```

Or add to your Python path permanently:
```python
import sys
sys.path.append('/path/to/binaryninja/python')
```

## Quick Start

### Run All Tests

```bash
cd /path/to/binaryninja-api
./run_architecture_tests.sh
```

**Expected output (all passing):**
```
Binary Ninja found

Starting architecture tests...

========================================
Running: RISC-V JALR Branch Detection
========================================
Testing RISC-V JALR branch detection (8 tests)...
PASS Test 1: jalr x5, x10, 0 - indirect call
PASS Test 2: jalr x10, x5, 4 - indirect call
...
Results: 8 passed, 0 failed out of 8 tests

PASS: RISC-V JALR Branch Detection

========================================
Running: MIPS64R6 JALR Return Recognition
========================================
...

========================================
Test Summary
========================================
Total test suites: 4
Passed: 4
Failed: 0

All tests passed!
```

## Individual Test Suites

### 1. RISC-V JALR Branch Detection

**Test:** `arch/riscv/test_riscv_jalr.py`
**Issue:** #6273
**What it validates:** JALR instructions correctly report branch types

```bash
python3 arch/riscv/test_riscv_jalr.py
```

**Expected results:**

| Test | Instruction | Expected Branch Type | IL Operation |
|------|-------------|---------------------|--------------|
| 1 | `jalr x5, x10, 0` | IndirectBranch | LLIL_CALL |
| 2 | `jalr x10, x5, 4` | IndirectBranch | LLIL_CALL |
| 3 | `jalr x3, x7, 8` | IndirectBranch | LLIL_CALL |
| 4 | `jalr x0, x1, 0` (ret) | FunctionReturn | LLIL_RET |
| 5 | `jalr x0, x5, 0` | UnresolvedBranch | LLIL_JUMP |
| 6 | `jalr x0, x10, 8` | UnresolvedBranch | LLIL_JUMP |
| 7 | `jalr x1, x1, 0` | FunctionReturn | LLIL_RET |
| 8 | `jalr x2, x2, 0` | IndirectBranch | LLIL_CALL |

**Success criteria:**
- All 8 branch detection tests pass
- All 3 IL lifting tests pass
- Exit code: 0

**Failure example:**
```
FAIL Test 1: jalr x5, x10, 0 - indirect call
  Expected: IndirectBranch
  Actual: (no branch detected)
  Instruction bytes: 93825000
```

### 2. MIPS64R6 JALR Return Recognition

**Test:** `arch/mips/test_mips64r6_jalr.py`
**Issue:** #7355
**What it validates:** MIPS R6 JALR variants correctly detected

```bash
python3 arch/mips/test_mips64r6_jalr.py
```

**Expected results:**

| Test | Instruction | Expected Branch Type | IL Operation |
|------|-------------|---------------------|--------------|
| 1 | `jalr $zero, $ra` | FunctionReturn | LLIL_RET |
| 2 | `jalr $zero, $t0` | UnresolvedBranch | LLIL_JUMP |
| 3 | `jalr $zero, $t1` | UnresolvedBranch | LLIL_JUMP |
| 4 | `jalr $ra, $t0` | IndirectBranch | LLIL_CALL |
| 5 | `jalr $ra, $t1` | IndirectBranch | LLIL_CALL |
| 6 | `jalr $ra, $t2` | IndirectBranch | LLIL_CALL |
| 7 | `jalr.hb $zero, $ra` | FunctionReturn | LLIL_RET |
| 8 | `jalr.hb $zero, $t0` | UnresolvedBranch | LLIL_JUMP |

**Success criteria:**
- All 8 branch detection tests pass
- All 3 IL lifting tests pass
- Exit code: 0

### 3. x86 BEXTR Semantic Lifting

**Test:** `arch/x86/test_bextr_lifting.py`
**Issue:** #6287
**What it validates:** BEXTR generates semantic IL, not intrinsic

```bash
python3 arch/x86/test_bextr_lifting.py
```

**Expected results:**

| Test | Instruction | Must Contain | Must NOT Contain |
|------|-------------|--------------|------------------|
| 1 | `bextr eax, ebx, ecx` | LLIL_LSR, LLIL_AND | LLIL_INTRINSIC |
| 2 | `bextr edx, [rsi], edi` | LLIL_LOAD, LLIL_AND | LLIL_INTRINSIC |
| 3 | `bextr rax, rbx, rcx` | LLIL_LSR, LLIL_AND | LLIL_INTRINSIC |

**IL Pattern Expected:**
```
LLIL_SET_REG.d(eax,
    LLIL_AND.d(
        LLIL_LSR.d(ebx, <start>),
        <mask>))
```

**Success criteria:**
- No LLIL_INTRINSIC in BEXTR lifting
- IL contains shift and mask operations
- All 6 tests pass
- Exit code: 0

**Failure example:**
```
FAIL Test 1: bextr eax, ebx, ecx
  BEXTR still lifted as intrinsic (opaque)
  IL: LLIL_SET_REG.d(eax,LLIL_INTRINSIC([],__bextr,[...]))
```

### 4. ARM64 Atomic MIN/MAX Intrinsics

**Test:** `arch/arm64/test_atomic_minmax.py`
**Issue:** #6599
**What it validates:** All LSE atomic MIN/MAX intrinsics recognized

```bash
python3 arch/arm64/test_atomic_minmax.py
```

**Expected results (30 intrinsic tests + 4 output tests = 34 total):**

| Intrinsic | Variant | Description |
|-----------|---------|-------------|
| `__ldsmax` | Word, A/L/AL | Atomic signed maximum |
| `__ldsmaxb` | Byte, A/L/AL | Atomic signed maximum (byte) |
| `__ldsmaxh` | Halfword, A/L/AL | Atomic signed maximum (halfword) |
| `__ldsmin` | Word, A/L/AL | Atomic signed minimum |
| `__ldsminb` | Byte, A/L/AL | Atomic signed minimum (byte) |
| `__ldsminh` | Halfword, A/L/AL | Atomic signed minimum (halfword) |
| `__ldumax` | Word, A/L/AL | Atomic unsigned maximum |
| `__ldumaxb` | Byte, A/L/AL | Atomic unsigned maximum (byte) |
| `__ldumaxh` | Halfword, A/L/AL | Atomic unsigned maximum (halfword) |
| `__ldumin` | Word, A/L/AL | Atomic unsigned minimum |
| `__lduminb` | Byte, A/L/AL | Atomic unsigned minimum (byte) |
| `__lduminh` | Halfword, A/L/AL | Atomic unsigned minimum (halfword) |
| `__stsmax` | Word, L | Store-only signed maximum |
| `__stsmaxb` | Byte, L | Store-only signed maximum (byte) |
| `__stsmaxh` | Halfword, L | Store-only signed maximum (halfword) |
| `__stsmin` | Word, L | Store-only signed minimum |
| `__stsminb` | Byte, L | Store-only signed minimum (byte) |
| `__stsminh` | Halfword, L | Store-only signed minimum (halfword) |
| `__stumax` | Word, L | Store-only unsigned maximum |
| `__stumaxb` | Byte, L | Store-only unsigned maximum (byte) |
| `__stumaxh` | Halfword, L | Store-only unsigned maximum (halfword) |
| `__stumin` | Word, L | Store-only unsigned minimum |
| `__stuminb` | Byte, L | Store-only unsigned minimum (byte) |
| `__stuminh` | Halfword, L | Store-only unsigned minimum (halfword) |

**Success criteria:**
- All 30 intrinsic recognition tests pass
- LD* variants have output registers (SET_REG)
- ST* variants do NOT have output registers
- All 34 tests pass
- Exit code: 0

## Troubleshooting

### Binary Ninja Not Found

**Error:**
```
ERROR: Binary Ninja not found. This test requires Binary Ninja to be installed.
```

**Solution:**
1. Verify Binary Ninja is installed
2. Add Binary Ninja Python module to PYTHONPATH
3. Run: `python3 -c "import binaryninja; print('OK')"`

### Architecture Not Found

**Error:**
```
ERROR: RISC-V architecture not found
```

**Possible causes:**
- Using Binary Ninja free edition (lacks architecture plugins)
- Architecture plugin not built
- Using incompatible Binary Ninja version

**Solution:**
- Use Binary Ninja Commercial/Personal/Enterprise
- Rebuild architecture plugins
- Check Binary Ninja version compatibility

### Test Failures After Changes

If tests fail after modifications:

1. **Verify fix is implemented:**
   ```bash
   cd /path/to/binaryninja-api
   git log --oneline | grep -E "JALR|BEXTR|atomic"
   ```

2. **Check branch:**
   ```bash
   git branch -a | grep improve-processor
   ```

3. **Rebuild architecture:**
   ```bash
   cd arch/<architecture>
   mkdir -p build && cd build
   cmake .. && make -j$(nproc)
   ```

4. **Verify Binary Ninja sees updated plugins:**
   - Close Binary Ninja completely
   - Copy built plugins to Binary Ninja plugins directory
   - Restart Binary Ninja

### License Issues

**Error:**
```
Binary Ninja license not found or expired
```

**Solution:**
- Binary Ninja trial: 14 days from first activation
- Register for trial at: https://binary.ninja/free
- Or use existing Commercial/Personal/Enterprise license

## Advanced Testing

### Manual IL Inspection

To manually inspect IL for a specific instruction:

```python
import binaryninja
from binaryninja import binaryview

# Create binary view with instruction
instr_bytes = b'\x93\x82\x50\x00'  # jalr x5, x10, 0
bv = binaryview.BinaryView.new(instr_bytes)

# Add function and get IL
arch = binaryninja.Architecture['riscv']
bv.platform = arch.standalone_platform
bv.add_function(0)

# Print IL
for func in bv.functions:
    for block in func.lifted_il:
        for il in block:
            print(il)
```

### Testing with Real Binaries

Apply fixes to real-world binaries:

**RISC-V:**
```bash
# Analyze RISC-V binary with indirect calls
binaryninja path/to/riscv_binary
# Check: View -> Graphs -> Call Graph
# Should show indirect calls properly connected
```

**MIPS64R6:**
```bash
# Analyze MIPS R6 binary
binaryninja path/to/mips64r6_binary
# Check: Function boundaries should be correct
# Returns should be properly identified
```

**x86 BEXTR:**
```bash
# Analyze x86 binary with BMI instructions
binaryninja path/to/x86_binary_with_bmi
# Check: HLIL should show bit extraction logic
# Should NOT show intrinsic calls for BEXTR
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Architecture Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Binary Ninja
        run: |
          # Requires Binary Ninja license stored in secrets
          wget https://cdn.binary.ninja/installers/binaryninja_linux.zip
          unzip binaryninja_linux.zip

      - name: Configure Python Path
        run: echo "PYTHONPATH=$PWD/binaryninja/python" >> $GITHUB_ENV

      - name: Run Tests
        run: ./run_architecture_tests.sh
```

**Note:** Requires Binary Ninja license for CI.

## Expected Test Execution Time

| Test Suite | Estimated Time |
|------------|---------------|
| RISC-V JALR | 1-3 seconds |
| MIPS64R6 JALR | 1-3 seconds |
| x86 BEXTR | 1-3 seconds |
| ARM64 Atomics | 2-4 seconds |
| **Total** | **5-15 seconds** |

## Reporting Issues

If tests fail unexpectedly:

1. **Capture test output:**
   ```bash
   ./run_architecture_tests.sh 2>&1 | tee test_results.log
   ```

2. **Include in bug report:**
   - Binary Ninja version
   - Operating system
   - Test output
   - Architecture build status
   - Git commit hash

3. **Submit to:**
   - Binary Ninja issue tracker (for core issues)
   - This repository (for plugin issues)

## Success Checklist

- [ ] Binary Ninja Python API accessible
- [ ] All architecture plugins built
- [ ] RISC-V tests pass (11/11)
- [ ] MIPS64R6 tests pass (11/11)
- [ ] x86 BEXTR tests pass (6/6)
- [ ] ARM64 atomic tests pass (34/34)
- [ ] Total: 62/62 tests pass
- [ ] Exit code: 0

---

**Last Updated:** 2025-11-15
**Binary Ninja Version Tested:** 3.5+
**Contact:** See repository for maintainer information
