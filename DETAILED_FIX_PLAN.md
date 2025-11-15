# Detailed Fix Plan for Critical Architecture Bugs

---

## FIX #1: MIPS64R6 Return Instruction Recognition

### Code Location
- **Branch Detection**: `/home/user/binaryninja-api/arch/mips/arch_mips.cpp` lines 383-415
- **Instruction Enum**: `/home/user/binaryninja-api/arch/mips/mips/mips.h` lines 36-770
- **IL Generation**: `/home/user/binaryninja-api/arch/mips/il.cpp` lines 1436-1442

### Root Cause Details
MIPS64R6 changed the instruction encoding for return instructions. The current code only checks for `MIPS_JR` and `MIPS_JR_HB`, which are the MIPS standard encodings. MIPS64R6 may encode returns differently or use different instruction variants.

### Step-by-Step Fix

**Step 1: Update Instruction Enum**
Location: `/home/user/binaryninja-api/arch/mips/mips/mips.h`
After line 269 (MIPS_JR), add:
```cpp
// MIPS64R6 variants
MIPS_JR_R6,      // MIPS64R6 return instruction variant
MIPS_JR_HB_R6,   // MIPS64R6 return with hazard barrier
```

**Step 2: Update Branch Detection**
Location: `/home/user/binaryninja-api/arch/mips/arch_mips.cpp` lines 384-390
Change from:
```cpp
case MIPS_JR:
case MIPS_JR_HB:
```

To:
```cpp
case MIPS_JR:
case MIPS_JR_HB:
case MIPS_JR_R6:
case MIPS_JR_HB_R6:
```

**Step 3: Update IL Generation**
Location: `/home/user/binaryninja-api/arch/mips/il.cpp` lines 1436-1442
Change from:
```cpp
case MIPS_JR:
case MIPS_JR_HB:
```

To:
```cpp
case MIPS_JR:
case MIPS_JR_HB:
case MIPS_JR_R6:
case MIPS_JR_HB_R6:
```

**Step 4: Update Disassembler**
The underlying disassembler (in mips/mips/) needs to properly decode MIPS64R6 instructions. This requires consulting the MIPS64R6 specification to get the correct opcode values.

### Verification
Test with MIPS64R6 binaries containing return instructions:
```
Binary bytes: [MIPS64R6 JR encoding] → Should emit FunctionReturn branch
```

---

## FIX #2: RISC-V JALR Branch Emission

### Code Location
**File:** `/home/user/binaryninja-api/arch/riscv/src/lib.rs` lines 712-722

### Root Cause Details
The branch detection code only reports branches when the destination register (rd) is x0. However, JALR is ALWAYS an indirect branch:
- When rd == x0: indirect jump (no link)
- When rd != x0: indirect call (links return address)
- When rd == x1 && rs1 == x1 && imm == 0: function return

### Complete Fix
Replace the entire JALR case (lines 712-722) with:

```rust
Op::Jalr(ref i) => {
    // JALR is always a branch - it's an indirect jump/call
    let target_reg = i.rs1().id();  // Source register for branch target
    let link_reg = i.rd().id();     // Destination register (where return address is stored)
    let imm = i.imm();              // Immediate offset for branch target
    
    if link_reg == 0 {
        // jalr x0, rs, imm - indirect jump (no return address stored)
        if target_reg == 1 && imm == 0 {
            // jalr x0, x1, 0 - return (jump to x1 with no offset)
            res.add_branch(BranchKind::FunctionReturn);
        } else {
            // Other indirect jumps
            res.add_branch(BranchKind::Unresolved);
        }
    } else {
        // jalr rd, rs, imm where rd != x0 - indirect call
        // Return address (PC + 4) is stored in rd
        res.add_branch(BranchKind::Call);
    }
}
```

### Why This Works
1. **jalr x0, x1, 0**: `link_reg == 0 && target_reg == 1 && imm == 0` → FunctionReturn
2. **jalr x0, x5, 0**: `link_reg == 0 && target_reg != 1` → Unresolved
3. **jalr x1, x5, 0**: `link_reg != 0` → Call
4. **jalr x5, x10, 8**: `link_reg != 0` → Call

### Verification Tests
```rust
#[test]
fn test_jalr_branches() {
    // jalr x0, x1, 0 → return
    // jalr x0, x5, 0 → unresolved jump
    // jalr x1, x5, 0 → call
    // jalr x5, x10, 8 → call with offset
}
```

---

## FIX #3: ARM/Thumb Extra Arguments in Function Calls

### Code Location
- **Primary**: Function parameter detection logic (location TBD - needs investigation)
- **Supporting**: `/home/user/binaryninja-api/arch/armv7/arch_armv7.cpp` lines 2146-2177

### Root Cause Details
The calling convention definition itself is correct (4 integer argument registers). The issue likely lies in:
1. How parameters are inferred from function calls
2. Whether Thumb mode properly switches conventions
3. How stack spills are distinguished from actual arguments

### Investigation Steps

**Step 1: Identify the Real Bug Location**
1. Find test case from issue #6615
2. Analyze the function with "extra arguments"
3. Trace where parameters are being added to the function signature
4. Check if it's in the IL analysis, parameter inference, or type detection

**Step 2: Likely Code Paths**
Search for:
- `InferParametersFromCalls()`
- `AnalyzeFunction()`
- `GetStackOffset()`
- Parameter detection heuristics

**Step 3: Fix Strategy**
Most likely one of:
1. **Stack argument detection bug**: Code incorrectly counting stack slots as parameters
2. **Thumb mode issue**: Not properly switching to Thumb calling convention
3. **Register tracking bug**: Incorrectly identifying which registers are argument registers

### Example Fix Pattern
If the bug is stack argument detection:
```cpp
// WRONG: Counts all stack references as arguments
for (auto stackRef : function->GetStackReferences()) {
    parameters.push_back(stackRef);  // ← Adds all stack refs!
}

// CORRECT: Only count stack references that are actual arguments
for (auto stackRef : function->GetStackReferences()) {
    if (stackRef.offset < callingConvention->GetStackArgumentOffset()) {
        parameters.push_back(stackRef);  // Only true arguments
    }
}
```

---

## FIX #4: x86 BEXTR Instruction Lifting

### Code Location
- **Current handler**: `/home/user/binaryninja-api/arch/x86/il.cpp` line 4251 (falls through to intrinsic)
- **Intrinsic definition**: `/home/user/binaryninja-api/arch/x86/arch_x86_intrinsics.cpp` lines 3824-3825

### Root Cause Details
BEXTR has a complex third operand that encodes both START and LENGTH fields. The generic intrinsic handler doesn't decode these, resulting in less semantic information in the IL.

**BEXTR Encoding:**
```
BEXTR dst, src, control
control format: [LENGTH(15:8)][START(7:0)]
```

### Option A: Semantic IL Decomposition (Recommended)

Add a special case BEFORE line 4250 in il.cpp:

```cpp
case XED_ICLASS_BEXTR: {
    // BEXTR: Extract bits from source based on control register
    // BEXTR dst, src, control
    // control = [LENGTH(15:8)][START(7:0)]
    
    // For register control (most common):
    if (xed_operand_name(opTre) == XED_OPERAND_REG3) {
        // Read the control register value as LLIL
        ExprId controlExpr = ReadILOperand(il, xedd, addr, 2, opTreLen);
        
        // Extract START[7:0]
        ExprId startExpr = il.LowPart(1, controlExpr);
        
        // Extract LEN[15:8]
        ExprId lenExpr = il.LogicalShiftRight(2, 
            il.LowPart(2, controlExpr), 
            il.Const(1, 8));
        
        // src_value >> start
        ExprId src = ReadILOperand(il, xedd, addr, 1, opTwoLen);
        ExprId shiftedSrc = il.LogicalShiftRight(
            il.GetDefaultIntegerSize(),
            src,
            startExpr
        );
        
        // Create mask based on length
        // mask = (1 << len) - 1
        ExprId one = il.Const(1, 1);
        ExprId shifted_one = il.ShiftLeft(1, one, lenExpr);
        ExprId mask = il.Sub(1, shifted_one, one);
        
        // Final result: (src >> start) & mask
        ExprId result = il.And(il.GetDefaultIntegerSize(), shiftedSrc, mask);
        
        // Write to destination
        il.AddInstruction(WriteILOperand(il, xedd, addr, 0, 0, result));
    } else {
        // Immediate control value
        // ... handle immediate variant similarly
        LiftAsIntrinsic();
    }
    break;
}
```

### Option B: Enhanced Intrinsic (Simpler)

Modify `LiftAsIntrinsic()` to have special handling for BMI instructions:

```cpp
auto LiftAsIntrinsic = [& il, xi, xedd, addr, xedd_iForm, xedd_iClass] () mutable {
    // ... existing code ...
    
    // Special handling for BMI instructions with encoded immediate operands
    if (xedd_iClass == XED_ICLASS_BEXTR || 
        xedd_iClass == XED_ICLASS_BLSI ||
        xedd_iClass == XED_ICLASS_BLSR ||
        xedd_iClass == XED_ICLASS_BLSMSK) {
        // For BMI instructions, decode the immediate/control operand
        // Add decoded parameters showing the semantic breakdown
    }
    
    // ... rest of intrinsic lifting ...
};
```

### Testing

Create test case:
```cpp
// BEXTR r64, r/m64, r64
// Extract bits from source, start pos and length in control register
0x0f 0x3a 0xf7 0xc1 0x15 0x00  // bextr rax, rcx, r8
// Control in r8: 0x0308 = START=08, LEN=03 (extract 3 bits starting at position 8)
```

Expected IL: Shows shift and mask operations clearly

---

## Implementation Priority

### HIGH PRIORITY (Quick Fixes)
1. **RISC-V JALR (Fix #2)**: ~20 lines of Rust code, clear semantics
2. **MIPS64R6 (Fix #1)**: ~10 lines of code, requires ISA data

### MEDIUM PRIORITY (Moderate Effort)  
3. **x86 BEXTR (Fix #4)**: ~50-100 lines, well-defined semantics

### INVESTIGATION REQUIRED
4. **ARM/Thumb (Fix #3)**: Needs debugging to locate actual bug

---

## Testing Strategy

For each fix:

1. **Unit Tests**: Test the specific code path
2. **Integration Tests**: Test with real binaries containing these instructions
3. **Regression Tests**: Ensure existing functionality not broken

Example test structure for RISC-V:
```rust
#[test]
fn test_riscv_jalr_branch_detection() {
    let tests = vec![
        ("jalr x0, x1, 0", BranchKind::FunctionReturn),
        ("jalr x0, x5, 0", BranchKind::Unresolved),
        ("jalr x1, x5, 0", BranchKind::Call),
        ("jalr x5, x10, 8", BranchKind::Call),
    ];
    
    for (asm, expected_branch) in tests {
        let branch = analyze_instruction(asm);
        assert_eq!(branch.kind, expected_branch);
    }
}
```

