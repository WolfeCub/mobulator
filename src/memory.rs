use anyhow::Context;

use crate::utils::{BitExt, to_lowest_bit_set};

pub const MEM_SIZE: usize = 0xFFFF + 1;
pub const INTERRUPT_ENABLE: usize = 0xFFFF;
pub const INTERRUPT_FLAG: usize = 0xFF0F;

/// 0x0000 - 0x00FF: Boot ROM
/// 0x0000 - 0x3FFF: Game ROM Bank 0
/// 0x4000 - 0x7FFF: Game ROM Bank N
/// 0x8000 - 0x97FF: Tile RAM
/// 0x9800 - 0x9FFF: Background Map
/// 0xA000 - 0xBFFF: Cartridge RAM
/// 0xC000 - 0xDFFF: Working RAM
/// 0xE000 - 0xFDFF: Echo RAM
/// 0xFE00 - 0xFE9F: OAM (Object Atribute Memory)
/// 0xFEA0 - 0xFEFF: Unused
/// 0xFF00 - 0xFF7F: I/O Registers
/// 0xFF80 - 0xFFFE: High RAM Area
/// 0xFFFF: Interrupt Enabled Register
#[derive(Debug, Clone)]
pub struct Memory {
    pub memory: [u8; MEM_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            memory: [0; MEM_SIZE],
        }
    }
}

impl Memory {
    pub fn get_byte(&self, addr: u16) -> anyhow::Result<u8> {
        self.memory
            .get(usize::from(addr))
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Out of bounds memory access at {:x}", addr))
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        self.memory[usize::from(addr)] = value;
    }

    pub fn set_u16(&mut self, addr: u16, value: u16) {
        let [high, low] = value.to_be_bytes();
        self.memory[usize::from(addr)] = low;
        self.memory[usize::from(addr + 1)] = high;
    }

    pub fn load_instructions(&mut self, instructions: &[u8]) {
        self.memory[..instructions.len()].copy_from_slice(instructions);
    }

    pub fn interrupt_to_run(&mut self) -> anyhow::Result<Option<InterruptType>> {
        let ienable = self.memory[INTERRUPT_ENABLE];
        let iflag = self
            .memory
            .get_mut(INTERRUPT_FLAG)
            .context("IF out of bounds")?;

        let priority = to_lowest_bit_set(ienable & *iflag);

        if priority == 0 {
            return Ok(None);
        }

        let it: InterruptType = priority.try_into()?;
        iflag.set_bit(u32::from(it.bit()), false);

        Ok(Some(it))
    }
}

/// ┌────┬───┬───┬───┬──────┬──────┬─────┬───┬──────┐
/// │ IE │ 7 │ 6 │ 5 │  4   │  3   │  2  │ 1 │  0   │
/// ├────┼───┼───┼───┼──────┼──────┼─────┼───┼──────┤
/// │    │   │   │   │Joypad│Serial│Timer│LCD│VBlank│
/// └────┴───┴───┴───┴──────┴──────┴─────┴───┴──────┘
#[derive(Debug)]
pub struct InterruptByte<'a>(pub &'a mut u8);

impl<'a> InterruptByte<'a> {
    pub fn get_flag(&self, flag: InterruptType) -> bool {
        self.0.is_bit_set(u32::from(flag.bit()))
    }

    pub fn set_flag(&mut self, flag: InterruptType, val: bool) {
        self.0.set_bit(u32::from(flag.bit()), val);
    }
}

#[derive(Debug, Clone)]
pub enum InterruptType {
    Joypad,
    Serial,
    Timer,
    LCD,
    VBlank,
}

impl InterruptType {
    pub fn bit(&self) -> u8 {
        match self {
            InterruptType::Joypad => 4,
            InterruptType::Serial => 3,
            InterruptType::Timer => 2,
            InterruptType::LCD => 1,
            InterruptType::VBlank => 0,
        }
    }

    pub fn addr(&self) -> u16 {
        match self {
            InterruptType::Joypad => 0x60,
            InterruptType::Serial => 0x56,
            InterruptType::Timer => 0x50,
            InterruptType::LCD => 0x48,
            InterruptType::VBlank => 0x40,
        }
    }
}

impl TryFrom<u8> for InterruptType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            16 => InterruptType::Joypad,
            8 => InterruptType::Serial,
            4 => InterruptType::Timer,
            2 => InterruptType::LCD,
            1 => InterruptType::VBlank,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid value: '{value}' for InterruptType"
                ));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::Memory;

    #[test]
    fn memory_get_set_bytes() {
        let mut mem = Memory::default();

        let addr = 0xC7D1; // 0xC000 - 0xDFFF working mem
        let val = 0x1F;
        mem.set_byte(addr, val);

        assert_eq!(mem.get_byte(addr).expect("Unable to get byte"), val);
        assert_eq!(mem.memory[usize::from(addr)], val);
    }
}
