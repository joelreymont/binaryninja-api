/*
 * MSP430X Simple Test Program
 *
 * Minimal test with key MSP430X instructions for quick validation.
 *
 * Compile with: msp430-elf-gcc -mmcu=msp430f5529 -o test_simple.elf test_simple.c
 */

#include <msp430.h>

/*
 * Function using MOVA instruction
 */
void __attribute__((noinline)) use_mova(void) {
    __asm__ volatile("mova #0x12345, r15");
    __asm__ volatile("mova r15, r14");
}

/*
 * Function using CALLA instruction
 */
void __attribute__((noinline)) target_function(void) {
    __asm__ volatile("nop");
    __asm__ volatile("reta");
}

void __attribute__((noinline)) use_calla(void) {
    __asm__ volatile("calla #target_function");
}

/*
 * Function using PUSHM/POPM instructions
 */
void __attribute__((noinline)) use_pushm_popm(void) {
    // Save registers
    __asm__ volatile("pushm.a #4, r15");

    // Do some work
    __asm__ volatile("mova #0x1000, r15");
    __asm__ volatile("mova #0x2000, r14");

    // Restore registers
    __asm__ volatile("popm.a #4, r15");
}

/*
 * Function using rotate instructions
 */
void __attribute__((noinline)) use_rotates(void) {
    __asm__ volatile("rlam.a #2, r15");
    __asm__ volatile("rram.a #2, r14");
}

/*
 * Main function
 */
int main(void) {
    WDTCTL = WDTPW | WDTHOLD;

    use_mova();
    use_calla();
    use_pushm_popm();
    use_rotates();

    while(1);
    return 0;
}
