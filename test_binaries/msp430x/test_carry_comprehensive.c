/*
 * Comprehensive MSP430X Test Program
 * Tests carry operations, 20-bit addressing, and various instructions
 */

#include <stdint.h>

// Test ADDC - Add with carry (real instruction)
__attribute__((noinline))
uint16_t test_addc(uint16_t a, uint16_t b, uint16_t carry) {
    uint16_t result;

    // Set carry flag if carry != 0
    if (carry) {
        __asm__ volatile("setc");
    } else {
        __asm__ volatile("clrc");
    }

    // ADDC b, a -> a = a + b + carry
    __asm__ volatile(
        "addc %1, %0"
        : "+r"(a)
        : "r"(b)
    );

    return a;
}

// Test SUBC - Subtract with carry (real instruction)
__attribute__((noinline))
uint16_t test_subc(uint16_t a, uint16_t b, uint16_t carry) {
    uint16_t result;

    // Set carry flag if carry != 0
    if (carry) {
        __asm__ volatile("setc");
    } else {
        __asm__ volatile("clrc");
    }

    // SUBC b, a -> a = a - b - !carry
    __asm__ volatile(
        "subc %1, %0"
        : "+r"(a)
        : "r"(b)
    );

    return a;
}

// Test ADC - Add carry (emulated as ADDC #0, dst)
__attribute__((noinline))
uint16_t test_adc(uint16_t a, uint16_t carry) {
    // Set carry flag
    if (carry) {
        __asm__ volatile("setc");
    } else {
        __asm__ volatile("clrc");
    }

    // ADC a -> a = a + carry
    __asm__ volatile(
        "adc %0"
        : "+r"(a)
    );

    return a;
}

// Test SBC - Subtract carry (emulated as SUBC #0, dst)
__attribute__((noinline))
uint16_t test_sbc(uint16_t a, uint16_t carry) {
    // Set carry flag
    if (carry) {
        __asm__ volatile("setc");
    } else {
        __asm__ volatile("clrc");
    }

    // SBC a -> a = a - !carry
    __asm__ volatile(
        "sbc %0"
        : "+r"(a)
    );

    return a;
}

// Test multi-precision addition (uses ADDC)
__attribute__((noinline))
void test_multi_precision_add(uint32_t a, uint32_t b, uint32_t *result) {
    uint16_t a_low = a & 0xFFFF;
    uint16_t a_high = (a >> 16) & 0xFFFF;
    uint16_t b_low = b & 0xFFFF;
    uint16_t b_high = (b >> 16) & 0xFFFF;
    uint16_t r_low, r_high;

    __asm__ volatile(
        "clrc\n"              // Clear carry
        "add %2, %0\n"        // Add low words
        "addc %3, %1\n"       // Add high words with carry
        : "=r"(r_low), "=r"(r_high)
        : "r"(b_low), "r"(b_high), "0"(a_low), "1"(a_high)
    );

    *result = ((uint32_t)r_high << 16) | r_low;
}

// Test MOVA/ADDA/SUBA with 20-bit addressing
#ifdef __MSP430X_LARGE__
__attribute__((noinline))
void* test_mova(void *addr) {
    void *result;
    __asm__ volatile(
        "mova %1, %0"
        : "=r"(result)
        : "r"(addr)
    );
    return result;
}

__attribute__((noinline))
void* test_adda(void *addr, uint32_t offset) {
    void *result = addr;
    __asm__ volatile(
        "adda %1, %0"
        : "+r"(result)
        : "r"(offset)
    );
    return result;
}
#endif

// Test conditional jumps
__attribute__((noinline))
uint16_t test_conditional_jumps(uint16_t a, uint16_t b) {
    uint16_t result = 0;

    // Test JZ/JNZ
    if (a == 0) {
        result |= 0x01;
    }

    // Test JC (carry set)
    __asm__ volatile(
        "add %0, %0\n"        // Add to itself (may set carry)
        "jc 1f\n"             // Jump if carry
        "bis #0x02, %1\n"     // Set bit if no carry
        "1:\n"
        : "+r"(a), "+r"(result)
    );

    return result;
}

// Test various arithmetic and logical operations
__attribute__((noinline))
uint16_t test_arithmetic(uint16_t a, uint16_t b) {
    uint16_t result;

    // Test ADD
    result = a + b;

    // Test SUB
    result -= a;

    // Test AND
    result &= 0xFF;

    // Test XOR
    result ^= b;

    // Test BIC (bit clear)
    __asm__ volatile(
        "bic %1, %0"
        : "+r"(result)
        : "r"((uint16_t)0x0F)
    );

    return result;
}

// Main entry point
int main(void) {
    uint16_t x = 0x1234;
    uint16_t y = 0x5678;
    uint32_t result32;

    // Test carry operations
    x = test_addc(0x8000, 0x8000, 0);  // Should set carry
    x = test_subc(0x1000, 0x0FFF, 1);  // Subtract with carry
    x = test_adc(0xFFFF, 1);           // Add carry to max value
    x = test_sbc(0x1000, 0);           // Subtract borrow

    // Test multi-precision
    test_multi_precision_add(0x12345678, 0x87654321, &result32);

    // Test conditionals
    x = test_conditional_jumps(0, 0xFF);

    // Test arithmetic
    y = test_arithmetic(0xABCD, 0x1234);

    // Infinite loop to prevent exit
    while(1) {
        __asm__ volatile("nop");
    }

    return 0;
}
