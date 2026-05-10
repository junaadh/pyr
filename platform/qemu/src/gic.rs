use pyr_arch::platform::{
    MmioAccess, MmioAccessKind, MmioDevice, MmioDeviceError, MmioResult,
};

use crate::qemu;

const NUM_IRQS: usize = 64;
const NUM_ENABLE_REGS: usize = NUM_IRQS / 32;
const NUM_PENDING_REGS: usize = NUM_IRQS / 32;
const NUM_ACTIVE_REGS: usize = NUM_IRQS / 32;
const NUM_PRIORITY_REGS: usize = NUM_IRQS / 4;
const NUM_TARGET_REGS: usize = NUM_IRQS / 4;
const NUM_CONFIG_REGS: usize = NUM_IRQS / 16;

const SPURIOUS_IRQ: u32 = 1023;

pub struct GicState {
    dist_enabled: bool,
    cpu_enabled: bool,
    priority_mask: u8,
    binary_point: u8,

    enabled: [u32; NUM_ENABLE_REGS],
    pending: [u32; NUM_PENDING_REGS],
    active: [u32; NUM_ACTIVE_REGS],

    priority: [u32; NUM_PRIORITY_REGS],
    targets: [u32; NUM_TARGET_REGS],
    config: [u32; NUM_CONFIG_REGS],
}

impl GicState {
    pub const fn new() -> Self {
        Self {
            dist_enabled: false,
            cpu_enabled: false,
            priority_mask: 0xff,
            binary_point: 0,
            enabled: [0; NUM_ENABLE_REGS],
            pending: [0; NUM_PENDING_REGS],
            active: [0; NUM_ACTIVE_REGS],
            priority: [0xa0a0_a0a0; NUM_PRIORITY_REGS],
            targets: [0x0101_0101; NUM_TARGET_REGS],
            config: [0; NUM_CONFIG_REGS],
        }
    }
}

pub struct Gic {
    state: GicState,
}

impl Gic {
    pub const GICD_BASE: u64 = 0x0800_0000;
    pub const GICC_BASE: u64 = 0x0801_0000;
    pub const SIZE: u64 = 0x10000;

    const GICD_CTLR: u64 = 0x0000;
    const GICD_TYPER: u64 = 0x0004;
    const GICD_IIDR: u64 = 0x0008;

    const GICC_CTLR: u64 = 0x0000;
    const GICC_PMR: u64 = 0x0004;
    const GICC_BPR: u64 = 0x0008;
    const GICC_IAR: u64 = 0x000c;
    const GICC_EOIR: u64 = 0x0010;
    const GICC_RPR: u64 = 0x0014;
    const GICC_HPPIR: u64 = 0x0018;
    const GICC_IIDR: u64 = 0x00fc;

    pub const fn new() -> Self {
        Self {
            state: GicState::new(),
        }
    }

    fn is_dist(ipa: u64) -> bool {
        (Self::GICD_BASE..Self::GICD_BASE + Self::SIZE).contains(&ipa)
    }

    fn is_cpu(ipa: u64) -> bool {
        (Self::GICC_BASE..Self::GICC_BASE + Self::SIZE).contains(&ipa)
    }

    pub fn base_for(ipa: u64) -> Option<u64> {
        if Self::is_cpu(ipa) {
            Some(Self::GICC_BASE)
        } else if Self::is_dist(ipa) {
            Some(Self::GICD_BASE)
        } else {
            None
        }
    }

    pub fn inject_irq(&mut self, irq: u32) {
        if irq < NUM_IRQS as u32 {
            self.set_pending(irq);
        }
    }

    pub fn has_pending_irq(&self) -> bool {
        self.highest_pending().is_some()
    }

    fn emulate_dist(
        &mut self,
        access: MmioAccess,
    ) -> Result<MmioResult, MmioDeviceError> {
        match access.kind {
            MmioAccessKind::Read { .. } => self.read_dist(access.offset),
            MmioAccessKind::Write { value, .. } => {
                self.write_dist(access.offset, value)
            }
        }
    }

    fn emulate_cpu(
        &mut self,
        access: MmioAccess,
    ) -> Result<MmioResult, MmioDeviceError> {
        match access.kind {
            MmioAccessKind::Read { .. } => self.read_cpu(access.offset),
            MmioAccessKind::Write { value, .. } => {
                self.write_cpu(access.offset, value)
            }
        }
    }

    fn read_dist(&self, offset: u64) -> Result<MmioResult, MmioDeviceError> {
        let value = match offset {
            Self::GICD_CTLR => self.state.dist_enabled as u32,
            Self::GICD_TYPER => ((NUM_IRQS as u32 / 32) - 1) & 0x1f,
            Self::GICD_IIDR => 0x0102_0143,

            0x100..=0x17f => read_reg(&self.state.enabled, offset, 0x100)?,
            0x180..=0x1ff => read_reg(&self.state.enabled, offset, 0x180)?,
            0x200..=0x27f => read_reg(&self.state.pending, offset, 0x200)?,
            0x280..=0x2ff => read_reg(&self.state.pending, offset, 0x280)?,
            0x300..=0x37f => read_reg(&self.state.active, offset, 0x300)?,
            0x380..=0x3ff => read_reg(&self.state.active, offset, 0x380)?,
            0x400..=0x7ff => read_reg(&self.state.priority, offset, 0x400)?,
            0x800..=0xbff => read_reg(&self.state.targets, offset, 0x800)?,
            0xc00..=0xcff => read_reg(&self.state.config, offset, 0xc00)?,

            unknown => {
                qemu!("gicd: read unknown offset={unknown:#x}");
                return Err(MmioDeviceError::BadRegister);
            }
        };

        Ok(MmioResult::Read(value as u64))
    }

    fn write_dist(
        &mut self,

        offset: u64,

        value: u64,
    ) -> Result<MmioResult, MmioDeviceError> {
        let value = value as u32;

        match offset {
            Self::GICD_CTLR => {
                self.state.dist_enabled = value & 1 != 0;
                Ok(MmioResult::Done)
            }

            0x100..=0x17f => {
                or_reg(&mut self.state.enabled, offset, 0x100, value)?;
                Ok(MmioResult::Done)
            }

            0x180..=0x1ff => {
                and_not_reg(&mut self.state.enabled, offset, 0x180, value)?;
                Ok(MmioResult::Done)
            }

            0x200..=0x27f => {
                or_reg(&mut self.state.pending, offset, 0x200, value)?;
                Ok(MmioResult::Done)
            }

            0x280..=0x2ff => {
                and_not_reg(&mut self.state.pending, offset, 0x280, value)?;
                Ok(MmioResult::Done)
            }

            0x300..=0x37f => {
                or_reg(&mut self.state.active, offset, 0x300, value)?;
                Ok(MmioResult::Done)
            }

            0x380..=0x3ff => {
                and_not_reg(&mut self.state.active, offset, 0x380, value)?;
                Ok(MmioResult::Done)
            }

            0x400..=0x7ff => {
                write_reg(&mut self.state.priority, offset, 0x400, value)
            }

            0x800..=0xbff => {
                write_reg(&mut self.state.targets, offset, 0x800, value)
            }

            0xc00..=0xcff => {
                write_reg(&mut self.state.config, offset, 0xc00, value)
            }

            unknown => {
                qemu!(
                    "gicd write unknown offset={unknown:#x} value={value:#x}"
                );

                Err(MmioDeviceError::BadRegister)
            }
        }
    }

    fn read_cpu(&mut self, offset: u64) -> Result<MmioResult, MmioDeviceError> {
        let value = match offset {
            Self::GICC_CTLR => self.state.cpu_enabled as u32,
            Self::GICC_PMR => self.state.priority_mask as u32,
            Self::GICC_BPR => self.state.binary_point as u32,

            Self::GICC_IAR => {
                if let Some(irq) = self.highest_pending() {
                    self.clear_pending(irq);
                    self.set_active(irq);
                    irq
                } else {
                    SPURIOUS_IRQ
                }
            }

            Self::GICC_RPR => 0xff,
            Self::GICC_HPPIR => self.highest_pending().unwrap_or(SPURIOUS_IRQ),
            Self::GICC_IIDR => 0x0202_143b,

            0x00d0..=0x00fc => 0,

            unknown => {
                qemu!("gicc: read unknown offset={unknown:#x}");
                return Err(MmioDeviceError::BadRegister);
            }
        };

        Ok(MmioResult::Read(value as u64))
    }

    fn write_cpu(
        &mut self,
        offset: u64,
        value: u64,
    ) -> Result<MmioResult, MmioDeviceError> {
        let value = value as u32;

        match offset {
            Self::GICC_CTLR => {
                self.state.cpu_enabled = value & 1 != 0;
                Ok(MmioResult::Done)
            }

            Self::GICC_PMR => {
                self.state.priority_mask = value as u8;
                Ok(MmioResult::Done)
            }

            Self::GICC_BPR => {
                self.state.binary_point = value as u8;
                Ok(MmioResult::Done)
            }

            Self::GICC_EOIR => {
                let irq = value & 0x3ff;

                if irq < NUM_IRQS as u32 {
                    self.clear_active(irq);
                }

                Ok(MmioResult::Done)
            }

            0x00d0..=0x00fc => Ok(MmioResult::Done),

            unknown => {
                qemu!(
                    "gicc: write unknown offset={unknown:#x} value={value:#x}"
                );
                Err(MmioDeviceError::BadRegister)
            }
        }
    }

    fn highest_pending(&self) -> Option<u32> {
        if !self.state.dist_enabled || !self.state.cpu_enabled {
            return None;
        }

        (0..NUM_IRQS as u32)
            .find(|&irq| self.is_enabled(irq) && self.is_pending(irq))
    }

    fn reg_bit(irq: u32) -> Option<(usize, u32)> {
        if irq >= NUM_IRQS as u32 {
            return None;
        }

        Some(((irq / 32) as usize, 1u32 << (irq % 32)))
    }

    fn is_enabled(&self, irq: u32) -> bool {
        if let Some((idx, bit)) = Self::reg_bit(irq)
            && let Some(&enabled) = self.state.enabled.get(idx)
        {
            enabled & bit != 0
        } else {
            false
        }
    }

    fn is_pending(&self, irq: u32) -> bool {
        if let Some((idx, bit)) = Self::reg_bit(irq)
            && let Some(&pending) = self.state.pending.get(idx)
        {
            pending & bit != 0
        } else {
            false
        }
    }

    fn set_pending(&mut self, irq: u32) {
        if let Some((idx, bit)) = Self::reg_bit(irq)
            && let Some(pending) = self.state.pending.get_mut(idx)
        {
            *pending |= bit;
        }
    }

    fn clear_pending(&mut self, irq: u32) {
        if let Some((idx, bit)) = Self::reg_bit(irq)
            && let Some(pending) = self.state.pending.get_mut(idx)
        {
            *pending &= !bit;
        }
    }

    fn set_active(&mut self, irq: u32) {
        if let Some((idx, bit)) = Self::reg_bit(irq)
            && let Some(active) = self.state.active.get_mut(idx)
        {
            *active |= bit;
        }
    }

    fn clear_active(&mut self, irq: u32) {
        if let Some((idx, bit)) = Self::reg_bit(irq)
            && let Some(active) = self.state.active.get_mut(idx)
        {
            *active &= !bit;
        }
    }
}

impl MmioDevice for Gic {
    fn contains(ipa: u64) -> bool {
        Self::is_dist(ipa) || Self::is_cpu(ipa)
    }

    fn emulate(
        &mut self,
        access: MmioAccess,
    ) -> Result<MmioResult, MmioDeviceError> {
        if Self::is_dist(access.ipa) {
            self.emulate_dist(access)
        } else if Self::is_cpu(access.ipa) {
            self.emulate_cpu(access)
        } else {
            Err(MmioDeviceError::BadRegister)
        }
    }
}

fn read_reg(
    regs: &[u32],
    offset: u64,
    base: u64,
) -> Result<u32, MmioDeviceError> {
    let idx = ((offset - base) / 4) as usize;
    regs.get(idx).copied().ok_or(MmioDeviceError::BadRegister)
}

fn write_reg(
    regs: &mut [u32],
    offset: u64,
    base: u64,
    value: u32,
) -> Result<MmioResult, MmioDeviceError> {
    let idx = ((offset - base) / 4) as usize;
    let Some(slot) = regs.get_mut(idx) else {
        return Err(MmioDeviceError::BadRegister);
    };
    *slot = value;
    Ok(MmioResult::Done)
}

fn or_reg(
    regs: &mut [u32],
    offset: u64,
    base: u64,
    value: u32,
) -> Result<(), MmioDeviceError> {
    let idx = ((offset - base) / 4) as usize;
    let Some(slot) = regs.get_mut(idx) else {
        return Err(MmioDeviceError::BadRegister);
    };
    *slot |= value;
    Ok(())
}

fn and_not_reg(
    regs: &mut [u32],
    offset: u64,
    base: u64,
    value: u32,
) -> Result<(), MmioDeviceError> {
    let idx = ((offset - base) / 4) as usize;
    let Some(slot) = regs.get_mut(idx) else {
        return Err(MmioDeviceError::BadRegister);
    };
    *slot &= !value;
    Ok(())
}
