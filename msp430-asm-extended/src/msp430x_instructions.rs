use crate::operand::Operand;
use std::fmt;

/// MSP430X Address Instructions (MOVA, CMPA, ADDA, SUBA)
/// These operate on 20-bit addresses
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AddressInstruction {
    mnemonic: &'static str,
    source: Operand,
    destination: Operand,
}

impl AddressInstruction {
    pub fn new(mnemonic: &'static str, source: Operand, destination: Operand) -> Self {
        AddressInstruction {
            mnemonic,
            source,
            destination,
        }
    }

    pub fn mnemonic(&self) -> &'static str {
        self.mnemonic
    }

    pub fn source(&self) -> &Operand {
        &self.source
    }

    pub fn destination(&self) -> &Operand {
        &self.destination
    }

    pub fn size(&self) -> usize {
        let mut size = 2; // Base instruction word

        // Add size for source operand data
        match self.source {
            Operand::Indexed(_) | Operand::Indexed20(_) | Operand::Symbolic(_)
            | Operand::Symbolic20(_) | Operand::Immediate(_) | Operand::Immediate20(_)
            | Operand::Absolute(_) | Operand::Absolute20(_) => size += 2,
            _ => {}
        }

        // Add size for destination operand data
        match self.destination {
            Operand::Indexed(_) | Operand::Indexed20(_) | Operand::Symbolic(_)
            | Operand::Symbolic20(_) | Operand::Absolute(_) | Operand::Absolute20(_) => {
                size += 2
            }
            _ => {}
        }

        size
    }
}

impl fmt::Display for AddressInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}, {}", self.mnemonic, self.source, self.destination)
    }
}

/// CALLA - Call subroutine (20-bit address)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calla {
    destination: Operand,
}

impl Calla {
    pub fn new(destination: Operand) -> Self {
        Calla { destination }
    }

    pub fn destination(&self) -> &Operand {
        &self.destination
    }

    pub fn size(&self) -> usize {
        match self.destination {
            Operand::RegisterDirect(_) => 2,
            Operand::Indexed(_) | Operand::Indexed20(_) | Operand::Immediate(_)
            | Operand::Immediate20(_) | Operand::Symbolic(_) | Operand::Symbolic20(_)
            | Operand::Absolute(_) | Operand::Absolute20(_) => 4,
            _ => 2,
        }
    }
}

impl fmt::Display for Calla {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "calla {}", self.destination)
    }
}

/// Rotate/Shift Multiple (RRCM, RRAM, RLAM, RRUM)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotateMultiple {
    mnemonic: &'static str,
    count: u8,      // 1-4
    register: u8,   // Rdst
    is_address: bool, // .A or .W mode
}

impl RotateMultiple {
    pub fn new(mnemonic: &'static str, count: u8, register: u8, is_address: bool) -> Self {
        RotateMultiple {
            mnemonic,
            count,
            register,
            is_address,
        }
    }

    pub fn mnemonic(&self) -> &'static str {
        self.mnemonic
    }

    pub fn count(&self) -> u8 {
        self.count
    }

    pub fn register(&self) -> u8 {
        self.register
    }

    pub fn is_address(&self) -> bool {
        self.is_address
    }

    pub fn size(&self) -> usize {
        2 // Always 1 word
    }
}

impl fmt::Display for RotateMultiple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.is_address { ".a" } else { ".w" };
        write!(f, "{}{} #{}, r{}", self.mnemonic, suffix, self.count, self.register)
    }
}

/// PUSHM/POPM - Push/Pop multiple registers
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PushPopMultiple {
    mnemonic: &'static str,
    count: u8,      // Number of registers (1-16)
    register: u8,   // Starting register
    is_address: bool, // .A or .W mode
}

impl PushPopMultiple {
    pub fn new(mnemonic: &'static str, count: u8, register: u8, is_address: bool) -> Self {
        PushPopMultiple {
            mnemonic,
            count,
            register,
            is_address,
        }
    }

    pub fn mnemonic(&self) -> &'static str {
        self.mnemonic
    }

    pub fn count(&self) -> u8 {
        self.count
    }

    pub fn register(&self) -> u8 {
        self.register
    }

    pub fn is_address(&self) -> bool {
        self.is_address
    }

    pub fn size(&self) -> usize {
        2 // Always 1 word
    }
}

impl fmt::Display for PushPopMultiple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if self.is_address { ".a" } else { ".w" };
        write!(f, "{}{} #{}, r{}", self.mnemonic, suffix, self.count, self.register)
    }
}

/// BRA - Branch (20-bit address)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bra {
    destination: Operand,
}

impl Bra {
    pub fn new(destination: Operand) -> Self {
        Bra { destination }
    }

    pub fn destination(&self) -> &Operand {
        &self.destination
    }

    pub fn size(&self) -> usize {
        match self.destination {
            Operand::RegisterDirect(_) => 2,
            Operand::Indexed(_) | Operand::Indexed20(_) | Operand::Symbolic(_)
            | Operand::Symbolic20(_) | Operand::Absolute(_) | Operand::Absolute20(_) => 4,
            Operand::Immediate(_) | Operand::Immediate20(_) => 4,
            _ => 2,
        }
    }
}

impl fmt::Display for Bra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bra {}", self.destination)
    }
}

/// RETA - Return from subroutine (20-bit)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reta;

impl Reta {
    pub fn new() -> Self {
        Reta
    }

    pub fn size(&self) -> usize {
        2
    }
}

impl fmt::Display for Reta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reta")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_instruction_display() {
        let inst = AddressInstruction::new(
            "mova",
            Operand::Immediate20(0x12345),
            Operand::RegisterDirect(15),
        );
        assert_eq!(format!("{}", inst), "mova #0x12345, r15");
    }

    #[test]
    fn calla_display() {
        let inst = Calla::new(Operand::Immediate20(0x10000));
        assert_eq!(format!("{}", inst), "calla #0x10000");
    }

    #[test]
    fn rotate_multiple_display() {
        let inst = RotateMultiple::new("rlam", 2, 15, true);
        assert_eq!(format!("{}", inst), "rlam.a #2, r15");

        let inst = RotateMultiple::new("rram", 3, 14, false);
        assert_eq!(format!("{}", inst), "rram.w #3, r14");
    }

    #[test]
    fn pushm_display() {
        let inst = PushPopMultiple::new("pushm", 4, 15, true);
        assert_eq!(format!("{}", inst), "pushm.a #4, r15");
    }

    #[test]
    fn popm_display() {
        let inst = PushPopMultiple::new("popm", 3, 10, false);
        assert_eq!(format!("{}", inst), "popm.w #3, r10");
    }

    #[test]
    fn bra_display() {
        let inst = Bra::new(Operand::Absolute20(0x10000));
        assert_eq!(format!("{}", inst), "bra &0x10000");

        let inst = Bra::new(Operand::RegisterDirect(15));
        assert_eq!(format!("{}", inst), "bra r15");
    }

    #[test]
    fn reta_display() {
        let inst = Reta::new();
        assert_eq!(format!("{}", inst), "reta");
    }

    #[test]
    fn address_instruction_size() {
        let inst = AddressInstruction::new(
            "mova",
            Operand::RegisterDirect(14),
            Operand::RegisterDirect(15),
        );
        assert_eq!(inst.size(), 2);

        let inst = AddressInstruction::new(
            "mova",
            Operand::Immediate20(0x12345),
            Operand::RegisterDirect(15),
        );
        assert_eq!(inst.size(), 4);
    }
}
