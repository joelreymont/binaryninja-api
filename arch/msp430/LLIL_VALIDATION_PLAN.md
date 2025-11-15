# MSP430X LLIL Validation Plan

## Current Status

**Environment Limitation**: Binary Ninja Free (downloaded and tested) does not include the SDK, Python API, or separate core library required for programmatic testing. Runtime LLIL validation requires a commercial Binary Ninja license with SDK access.

**Downloaded**: Binary Ninja Free for Linux (extracted to /tmp/binaryninja/)
**Finding**: Free version is a monolithic binary without libbinaryninjacore.so or Python API

However, we can document expected behavior and create test infrastructure for future validation.

## What We CAN Validate (Without Binary Ninja)

### 1. Code Compiles ✅
The LLIL lifting code compiles without errors, which means:
- Type safety is correct
- IL builder API is used correctly
- No obvious syntax errors

### 2. Logic Review ✅
Manual review shows the implementations follow correct patterns:

**RETA** - 20-bit return:
```rust
il.ret(il.pop(3)).append();
```
- ✅ Uses 3-byte pop (correct for 20-bit)
- ✅ Returns the popped value

**PUSHM** - Push multiple:
```rust
for i in 0..count {
    let reg_num = end_reg.wrapping_sub(i);
    il.push(size, il.reg(size, reg)).append();
}
```
- ✅ Loops correct number of times
- ✅ Pushes in descending order (correct)
- ✅ Uses correct size based on .a/.w mode

**RLAM** - Shift left arithmetic:
```rust
let op = il.lsl(size, reg_val, shift_amount).with_flag_write(FlagWrite::All);
il.set_reg(size, reg, op).append();
```
- ✅ Uses LSL (logical shift left)
- ✅ Updates flags
- ✅ Writes result back to register

### 3. Consistency with Base MSP430 ✅
MSP430X instructions follow the same patterns as existing base MSP430 instructions:
- Register operations use `il.set_reg()`
- Memory operations use `il.load()` and `il.store()`
- Branches use `il.call()`, `il.ret()`, etc.
- Flag updates use `.with_flag_write()`

## What We CANNOT Validate (Without Binary Ninja)

### 1. Actual LLIL Output ❌
Cannot verify that our code generates the expected IL strings:
```
Expected: LLIL_RET(LLIL_POP.w())
Actual:   ??? (need Binary Ninja to check)
```

### 2. IL Semantics ❌
Cannot verify that the IL has correct semantics:
- Does PUSHM generate 4 separate PUSH operations or 1 combined?
- Are register sizes correct (2-byte vs 3-byte)?
- Are memory accesses correct?

### 3. Integration Testing ❌
Cannot verify the full pipeline:
1. Decode instruction bytes
2. Generate tokens for display
3. Lift to LLIL
4. Render in UI

## Validation Plan for When Binary Ninja is Available

### Phase 1: Basic LLIL Generation
Load the plugin and verify it generates IL without crashing:

```python
import binaryninja as bn

# Load a MSP430X binary
bv = bn.load("test_simple.elf")

# Get function containing MSP430X instructions
func = bv.get_function_at(0x4424)  # use_pushm_popm

# Get LLIL
llil = func.low_level_il

# Print each instruction's LLIL
for block in llil:
    for instr in block:
        print(f"{instr.address:x}: {instr}")
```

**Expected**: Should print IL without errors

### Phase 2: Validate Specific Instructions
Compare actual LLIL output to expected output:

```python
def test_reta_llil():
    # Create view with RETA instruction
    data = b"\x10\x01"  # RETA
    bv = create_test_view(data, arch="msp430")

    # Get LLIL at address 0
    llil = bv.get_llil_at(0)

    # Check it's a RET with POP
    assert llil.operation == bn.LowLevelILOperation.LLIL_RET
    assert llil.operand.operation == bn.LowLevelILOperation.LLIL_POP
    assert llil.operand.size == 3  # 20-bit = 3 bytes
```

### Phase 3: Validate Complex Instructions
Test instructions that generate multiple IL operations:

```python
def test_pushm_llil():
    # PUSHM.A #4, r15 should generate 4 PUSH operations
    data = b"\x3f\x14"
    bv = create_test_view(data, arch="msp430")

    llil = bv.get_llil_at(0)

    # Should see 4 PUSH operations
    assert count_push_operations(llil) == 4

    # Registers should be r15, r14, r13, r12 (descending)
    regs = get_pushed_registers(llil)
    assert regs == [15, 14, 13, 12]
```

### Phase 4: Integration Testing
Test full binaries compiled with GCC:

```python
def test_gcc_binary():
    # Load our GCC-compiled test
    bv = bn.load("test_simple.elf")

    # Find function with MSP430X instructions
    func = bv.get_function_at(0x4424)

    # Verify function decompiles without errors
    assert func.hlil is not None

    # Check specific instructions
    llil = func.low_level_il

    # At 0x4424: PUSHM.A #4, r15
    assert get_llil_at(llil, 0x4424).operation == LLIL_PUSH

    # At 0x442e: POPM.A #4, r15
    assert get_llil_at(llil, 0x442e).operation == LLIL_SET_REG
```

## Expected LLIL Output

Based on our implementation, here's what we expect:

### RETA (0x10 0x01)
```
LLIL_RET(LLIL_POP.w())
```
- POP should be 3 bytes for 20-bit mode

### PUSHM.A #4, r15 (0x3f 0x14)
```
LLIL_PUSH.w(LLIL_REG.w(r15))
LLIL_PUSH.w(LLIL_REG.w(r14))
LLIL_PUSH.w(LLIL_REG.w(r13))
LLIL_PUSH.w(LLIL_REG.w(r12))
```
- 4 separate PUSH operations
- Descending register order

### POPM.A #4, r15 (0x3c 0x16)
```
LLIL_SET_REG.w(r12,LLIL_POP.w())
LLIL_SET_REG.w(r13,LLIL_POP.w())
LLIL_SET_REG.w(r14,LLIL_POP.w())
LLIL_SET_REG.w(r15,LLIL_POP.w())
```
- 4 SET_REG with POP
- Ascending register order

### RLAM.A #2, r15 (0x4f 0x06)
```
LLIL_SET_REG.w(r15,LLIL_LSL.w(LLIL_REG.w(r15),LLIL_CONST.w(2)))
```
- Logical shift left by 2
- Flag updates

### RRAM.A #2, r14 (0x4e 0x05)
```
LLIL_SET_REG.w(r14,LLIL_ASR.w(LLIL_REG.w(r14),LLIL_CONST.w(2)))
LLIL_SET_FLAG(V,LLIL_CONST(0))
```
- Arithmetic shift right by 2
- V flag cleared
- C, N, Z flags updated

### CALLA #0x4414 (0xb0 0x13 0x14 0x44)
```
LLIL_CALL(LLIL_CONST_PTR(0x4414))
```
- Call to immediate address

### MOVA r15, r14 (0xce 0x0f)
```
LLIL_SET_REG.w(r14,LLIL_REG.w(r15))
```
- Register to register move
- 3-byte (20-bit) size

## Confidence Assessment

| Aspect | Confidence | Reason |
|--------|-----------|--------|
| **Code compiles** | HIGH | Verified |
| **Logic correct** | MEDIUM-HIGH | Manually reviewed, follows patterns |
| **LLIL strings** | MEDIUM | Can't verify without runtime |
| **IL semantics** | MEDIUM | Implementation looks right, but unproven |
| **Edge cases** | LOW | Need extensive testing |

## Next Steps (With Binary Ninja)

1. ✅ Create test infrastructure (done - see test_llil_msp430x.py)
2. ⚠️ Install Binary Ninja
3. ⚠️ Build and load MSP430 plugin
4. ⚠️ Run Phase 1 tests (basic IL generation)
5. ⚠️ Run Phase 2 tests (specific instruction validation)
6. ⚠️ Run Phase 3 tests (complex instructions)
7. ⚠️ Run Phase 4 tests (full binary integration)

## Summary

**What we know**: The code compiles, follows correct patterns, and is logically sound.

**What we don't know**: Whether the actual LLIL output matches expectations.

**To fully validate**: Need Binary Ninja installed to run runtime tests.

**Recommendation**: Implementation appears correct based on code review, but requires Binary Ninja for definitive validation.
