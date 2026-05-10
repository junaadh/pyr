use pyr_arch::platform::PhysicalInterruptController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QemuIrq(pub u32);

pub struct QemuGic;

impl PhysicalInterruptController for QemuGic {
    type Irq = QemuIrq;

    fn acknowledge() -> Self::Irq {
        let raw = read32(0x0801_0000 + 0x0c);
        QemuIrq(raw & 0x3ff)
    }

    fn complete(irq: Self::Irq) {
        if !Self::is_spurious(irq) {
            write32(0x0801_0000 + 0x10, irq.0);
        }
    }

    fn is_spurious(irq: Self::Irq) -> bool {
        irq.0 >= 1020
    }
}

fn read32(addr: usize) -> u32 {
    // SAFETY: `addr` is a valid GIC CPU-interface MMIO register on QEMU virt.
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

fn write32(addr: usize, value: u32) {
    // SAFETY: `addr` is a valid GIC CPU-interface MMIO register on QEMU virt.
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, value);
    }
}
