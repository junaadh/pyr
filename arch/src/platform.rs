pub trait Platform {
    fn early_init();

    fn early_putc(byte: u8);
    fn early_print(s: &str) {
        s.bytes().for_each(Self::early_putc);
    }
}
