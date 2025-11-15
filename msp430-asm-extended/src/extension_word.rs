use crate::decode_error::DecodeError;

/// Extension word format for MSP430X instructions
/// Format: 18xx xxxx xxxx xxxx (first nibble is always 1, second nibble is 8-F)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtensionWord {
    raw: u16,
}

impl ExtensionWord {
    /// Parse an extension word from raw bytes
    pub fn parse(data: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if data.len() < 2 {
            return Err(DecodeError::MissingOperand);
        }

        let (word_bytes, remaining) = data.split_at(2);
        let word = u16::from_le_bytes([word_bytes[0], word_bytes[1]]);

        // Extension word format: 0001 1xxx xxxx xxxx (high nibble is 1, second nibble 8-F)
        // This means the high 5 bits are 00011 (0x18xx - 0x1Fxx range)
        if (word & 0xF800) != 0x1800 {
            return Err(DecodeError::InvalidExtensionWord);
        }

        Ok((ExtensionWord { raw: word }, remaining))
    }

    /// Get the source extension bits (bits 0-6)
    /// These extend the source operand to 20 bits
    pub fn source_extension(&self) -> u8 {
        (self.raw & 0x0780) as u8 >> 7
    }

    /// Get the destination extension bits (bits 7-9)
    /// These extend the destination operand to 20 bits
    pub fn dest_extension(&self) -> u8 {
        (self.raw & 0x000F) as u8
    }

    /// Get AL bit (bit 6) - Address/Word mode for source
    /// false = Word (.W), true = Address (.A)
    pub fn source_is_address(&self) -> bool {
        (self.raw & 0x0040) != 0
    }

    /// Get ZC bit (bit 8) - Zero Carry
    pub fn zero_carry(&self) -> bool {
        (self.raw & 0x0100) != 0
    }

    /// Get repetition count (bits 0-3) for repeat instructions
    /// Note: actual count is value + 1
    pub fn repeat_count(&self) -> u8 {
        ((self.raw & 0x000F) + 1) as u8
    }

    /// Extend a 16-bit source value to 20 bits using extension bits
    pub fn extend_source(&self, value: u16) -> u32 {
        let ext = (self.source_extension() as u32) << 16;
        ext | (value as u32)
    }

    /// Extend a 16-bit destination value to 20 bits using extension bits
    pub fn extend_dest(&self, value: u16) -> u32 {
        let ext = (self.dest_extension() as u32) << 16;
        ext | (value as u32)
    }

    /// Get raw extension word value
    pub fn raw(&self) -> u16 {
        self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_extension_word() {
        let data = [0x00, 0x18]; // 0x1800
        let result = ExtensionWord::parse(&data);
        assert!(result.is_ok());
        let (ext, remaining) = result.unwrap();
        assert_eq!(ext.raw, 0x1800);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn parse_invalid_extension_word() {
        let data = [0x00, 0x10]; // 0x1000 - not in extension range
        let result = ExtensionWord::parse(&data);
        assert_eq!(result, Err(DecodeError::InvalidExtensionWord));
    }

    #[test]
    fn extension_bits() {
        let data = [0x8F, 0x18]; // 0x188F
        let (ext, _) = ExtensionWord::parse(&data).unwrap();

        // Source extension: bits 7-9 shifted right by 7
        // 0x188F & 0x0780 = 0x0080 >> 7 = 1
        assert_eq!(ext.source_extension(), 1);

        // Dest extension: bits 0-3
        // 0x188F & 0x000F = 0x000F = 15
        assert_eq!(ext.dest_extension(), 15);
    }

    #[test]
    fn extend_values() {
        let data = [0x45, 0x19]; // 0x1945
        let (ext, _) = ExtensionWord::parse(&data).unwrap();

        // Extend 0x1234 to 20 bits
        let extended = ext.extend_source(0x1234);
        // Source extension bits should affect upper nibble
        assert_eq!(extended & 0xFFFFF, (ext.source_extension() as u32) << 16 | 0x1234);
    }
}
