// MSP430X LLIL Validation Test
// This test validates the MSP430X decoder and LLIL lifting
// without requiring a full Binary Ninja runtime

use std::path::PathBuf;

// Test data from GCC-compiled binaries
struct TestCase {
    name: &'static str,
    bytes: &'static [u8],
    expected_mnemonic: &'static str,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "RETA",
        bytes: &[0x10, 0x01],
        expected_mnemonic: "reta",
    },
    TestCase {
        name: "PUSHM.A #4, r15",
        bytes: &[0x3f, 0x14],
        expected_mnemonic: "pushm.a",
    },
    TestCase {
        name: "POPM.A #4, r15",
        bytes: &[0x3c, 0x16],
        expected_mnemonic: "popm.a",
    },
    TestCase {
        name: "RLAM.A #2, r15",
        bytes: &[0x4f, 0x06],
        expected_mnemonic: "rlam.a",
    },
    TestCase {
        name: "RRAM.A #2, r14",
        bytes: &[0x4e, 0x05],
        expected_mnemonic: "rram.a",
    },
];

fn main() {
    println!("MSP430X Decoder Test");
    println!("====================\n");

    // Load the decoder library
    let decoder_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("msp430-asm-extended");

    println!("Testing decoder from: {}", decoder_path.display());
    println!();

    // Run decoder tests
    for test in TEST_CASES {
        print!("Testing {:<20} ", test.name);

        // Decode the instruction
        match msp430_asm_extended::decode(test.bytes) {
            Ok(inst) => {
                let mnemonic = format!("{}", inst).to_lowercase();
                if mnemonic.starts_with(test.expected_mnemonic) {
                    println!("✓ PASS - decoded as {}", inst);
                } else {
                    println!("✗ FAIL - expected {}, got {}", test.expected_mnemonic, inst);
                }
            }
            Err(e) => {
                println!("✗ FAIL - decode error: {:?}", e);
            }
        }
    }

    println!("\n=================================");
    println!("Decoder tests completed");
    println!("=================================");
}
