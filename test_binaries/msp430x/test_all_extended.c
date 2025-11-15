/*
 * MSP430X Extended Instructions Test Program
 *
 * This program contains examples of all MSP430X extended instructions
 * for testing disassembly and lifting in Binary Ninja.
 *
 * Compile with: msp430-elf-gcc -mmcu=msp430f5529 -o test_all_extended.elf test_all_extended.c
 */

#include <msp430.h>

/*
 * Test MOVA - Move Address (20-bit)
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_mova(void) {
    // MOVA immediate to register
    __asm__ volatile("mova #0x12345, r15");

    // MOVA register to register
    __asm__ volatile("mova r14, r15");

    // MOVA absolute to register
    __asm__ volatile("mova &0x10000, r13");

    // MOVA register to absolute
    __asm__ volatile("mova r12, &0x10000");

    // MOVA indexed to register
    __asm__ volatile("mova 0x100(r10), r11");
}

/*
 * Test CMPA - Compare Address (20-bit)
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_cmpa(void) {
    // CMPA immediate to register
    __asm__ volatile("cmpa #0x10000, r15");

    // CMPA register to register
    __asm__ volatile("cmpa r14, r15");

    // CMPA absolute to register
    __asm__ volatile("cmpa &0x10000, r13");

    // CMPA indexed to register
    __asm__ volatile("cmpa 0x100(r10), r11");
}

/*
 * Test ADDA - Add to Address (20-bit)
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_adda(void) {
    // ADDA immediate to register
    __asm__ volatile("adda #0x1000, r15");

    // ADDA register to register
    __asm__ volatile("adda r14, r15");

    // ADDA with constant generator (special behavior)
    // ADDA R3, Rdst increments Rdst by 2
    __asm__ volatile("adda r3, r12");

    // ADDA R2, Rdst increments Rdst by 4
    __asm__ volatile("adda r2, r11");
}

/*
 * Test SUBA - Subtract from Address (20-bit)
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_suba(void) {
    // SUBA immediate from register
    __asm__ volatile("suba #0x1000, r15");

    // SUBA register from register
    __asm__ volatile("suba r14, r15");

    // SUBA with constant generator (special behavior)
    // SUBA R3, Rdst decrements Rdst by 2
    __asm__ volatile("suba r3, r12");

    // SUBA R2, Rdst decrements Rdst by 4
    __asm__ volatile("suba r2, r11");
}

/*
 * Test CALLA - Call Subroutine (20-bit address)
 * Opcode: 13xx range
 */
void __attribute__((noinline)) test_calla_target(void) {
    // Just a target function for CALLA
    __asm__ volatile("nop");
}

void __attribute__((noinline)) test_calla(void) {
    // CALLA immediate address
    __asm__ volatile("calla #0x10000");

    // CALLA register (indirect)
    __asm__ volatile("calla r15");

    // CALLA indexed
    __asm__ volatile("calla 0x100(r14)");

    // CALLA to function
    __asm__ volatile("calla #test_calla_target");
}

/*
 * Test BRA - Branch (20-bit address)
 * Emulated instruction using MOVA
 */
void __attribute__((noinline)) test_bra(void) {
    // BRA immediate
    __asm__ volatile("bra #0x10000");

    // BRA register
    __asm__ volatile("bra r15");

    // BRA indexed
    __asm__ volatile("bra 0x100(r14)");
}

/*
 * Test RETA - Return from Subroutine (20-bit)
 * Restores 20-bit PC and adjusts SP by 4
 */
void __attribute__((noinline)) test_reta(void) {
    __asm__ volatile("reta");
}

/*
 * Test RRCM - Rotate Right through Carry, Multiple bits
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_rrcm(void) {
    // RRCM.A - Address mode (20-bit)
    // Rotate r15 right 1 bit through carry
    __asm__ volatile("rrcm.a #1, r15");

    // Rotate r14 right 2 bits through carry
    __asm__ volatile("rrcm.a #2, r14");

    // Rotate r13 right 3 bits through carry
    __asm__ volatile("rrcm.a #3, r13");

    // Rotate r12 right 4 bits through carry
    __asm__ volatile("rrcm.a #4, r12");

    // RRCM.W - Word mode (16-bit)
    __asm__ volatile("rrcm.w #2, r11");
}

/*
 * Test RRAM - Rotate Right Arithmetic, Multiple bits
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_rram(void) {
    // RRAM.A - Address mode (20-bit)
    __asm__ volatile("rram.a #1, r15");
    __asm__ volatile("rram.a #2, r14");
    __asm__ volatile("rram.a #3, r13");
    __asm__ volatile("rram.a #4, r12");

    // RRAM.W - Word mode (16-bit)
    __asm__ volatile("rram.w #2, r11");
}

/*
 * Test RLAM - Rotate Left Arithmetic, Multiple bits
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_rlam(void) {
    // RLAM.A - Address mode (20-bit)
    __asm__ volatile("rlam.a #1, r15");
    __asm__ volatile("rlam.a #2, r14");
    __asm__ volatile("rlam.a #3, r13");
    __asm__ volatile("rlam.a #4, r12");

    // RLAM.W - Word mode (16-bit)
    __asm__ volatile("rlam.w #2, r11");
}

/*
 * Test RRUM - Rotate Right Unsigned, Multiple bits
 * Opcode: 0xxx range
 */
void __attribute__((noinline)) test_rrum(void) {
    // RRUM.A - Address mode (20-bit)
    __asm__ volatile("rrum.a #1, r15");
    __asm__ volatile("rrum.a #2, r14");
    __asm__ volatile("rrum.a #3, r13");
    __asm__ volatile("rrum.a #4, r12");

    // RRUM.W - Word mode (16-bit)
    __asm__ volatile("rrum.w #2, r11");
}

/*
 * Test PUSHM - Push Multiple Registers
 * Opcode: 14xx range
 */
void __attribute__((noinline)) test_pushm(void) {
    // PUSHM.A - Push multiple registers in address mode (20-bit)
    // Push 4 registers starting from r15 (r15, r14, r13, r12)
    __asm__ volatile("pushm.a #4, r15");

    // Push 3 registers starting from r10 (r10, r9, r8)
    __asm__ volatile("pushm.a #3, r10");

    // PUSHM.W - Push multiple registers in word mode (16-bit)
    // Push 2 registers starting from r7 (r7, r6)
    __asm__ volatile("pushm.w #2, r7");
}

/*
 * Test POPM - Pop Multiple Registers
 * Opcode: 14xx range
 */
void __attribute__((noinline)) test_popm(void) {
    // POPM.A - Pop multiple registers in address mode (20-bit)
    // Pop 2 registers into r7, r6
    __asm__ volatile("popm.a #2, r7");

    // Pop 3 registers into r10, r9, r8
    __asm__ volatile("popm.a #3, r10");

    // Pop 4 registers into r15, r14, r13, r12
    __asm__ volatile("popm.a #4, r15");

    // POPM.W - Pop multiple registers in word mode (16-bit)
    __asm__ volatile("popm.w #2, r7");
}

/*
 * Test mixed MSP430 and MSP430X instructions
 * Verify that base MSP430 instructions still work
 */
void __attribute__((noinline)) test_mixed(void) {
    // Base MSP430 instruction
    __asm__ volatile("mov #0x5A80, r15");

    // MSP430X extended instruction
    __asm__ volatile("mova #0x12345, r14");

    // Base MSP430 instruction
    __asm__ volatile("add r14, r15");

    // MSP430X extended instruction
    __asm__ volatile("adda r14, r13");

    // Base MSP430 instruction
    __asm__ volatile("cmp #0x100, r15");

    // MSP430X extended instruction
    __asm__ volatile("cmpa #0x10000, r14");
}

/*
 * Main function - calls all test functions
 */
int main(void) {
    // Stop watchdog timer
    WDTCTL = WDTPW | WDTHOLD;

    // Call all test functions
    test_mova();
    test_cmpa();
    test_adda();
    test_suba();
    test_calla();
    test_bra();
    test_rrcm();
    test_rram();
    test_rlam();
    test_rrum();
    test_pushm();
    test_popm();
    test_mixed();

    // Infinite loop
    while(1) {
        __asm__ volatile("nop");
    }

    return 0;
}
