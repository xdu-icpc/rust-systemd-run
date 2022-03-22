fn main() {
    let x = vec![1u8; 128 << 20];
    // attempt to prevent rustc from optimizing x away
    unsafe {
        std::arch::asm!("nop /* {0} */", in(reg) &x);
    }
}
