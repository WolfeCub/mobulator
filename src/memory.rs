pub const MEM_SIZE: usize = 0xFFFF + 1;

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

    pub fn load_instructions(&mut self, instructions: &[u8]) {
        self.memory[..instructions.len()].copy_from_slice(instructions);
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
