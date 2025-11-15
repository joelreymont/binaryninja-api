# msp430-asm-extended

Extended fork of msp430-asm with MSP430X (20-bit) instruction support.

## Overview

This is an extended version of the msp430-asm disassembly library that adds support for MSP430X extended instructions including:

- 20-bit addressing support
- Extension word parsing
- Address instructions (MOVA, CMPA, ADDA, SUBA)
- Extended control flow (CALLA, BRA, RETA)
- Multi-bit rotates/shifts (RRCM, RRAM, RLAM, RRUM)
- Multiple register stack operations (PUSHM, POPM)

## Status

This extended version is based on msp430-asm v0.3.0 and adds MSP430X instruction support for use in Binary Ninja's MSP430 architecture module.

## Upstream

Original repository: https://github.com/jrozner/msp430-asm

## License

MIT (same as upstream)

## Changes from Upstream

1. Added extension word parsing
2. Added 20-bit operand support
3. Added MSP430X instruction variants
4. Extended decoder for 0xxx and 13xx/14xx opcode ranges
5. Added .A (address) and .W (word) width support
6. Added tests for all extended instructions

## Usage

```rust
use msp430_asm_extended::decode;

let data = &[0x00, 0x00, 0x45, 0x23, 0x01]; // MOVA example
match decode(data) {
    Ok(inst) => println!("{}", inst),
    Err(e) => println!("Decode error: {:?}", e),
}
```

## Building

```bash
cargo build
cargo test
```

## Integration with Binary Ninja

Update `arch/msp430/Cargo.toml`:

```toml
[dependencies]
msp430-asm-extended = { path = "../../msp430-asm-extended" }
```

## Documentation

See MSP430X_IMPLEMENTATION_PLAN.md in the parent directory for implementation details.

## References

- MSP430X CPU User's Guide (SLAU391F)
- MSP430x2xx Family User's Guide (SLAU144J)
- MSP430x5xx/MSP430x6xx Family User's Guide (SLAU208G)
