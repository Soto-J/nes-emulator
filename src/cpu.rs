use crate::opcodes;
use std::collections::HashMap;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF], // Size 65535
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

trait Mem {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let high = self.mem_read(addr + 1) as u16;
        let low = self.mem_read(addr) as u16;

        // u16::from_le_bytes([low as u8, high as u8])
        (high << 8) | low
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        // Rust Endian support
        // let [low, high] = data.to_le_bytes();

        // Get FIRST 8 bits EX: 0001 0010 0011 0100 >> 8 = 0000 0000 0001 0010
        let high = (data >> 8) as u8;
        // Get LAST 8 bits. EX: 0001 0010 0011 0100 & 1111 1111 = 0011 0100
        let low = (data & 0xFF) as u8;

        // Nes uses Little Endian method: [0011 0100, 0001 0010]
        self.mem_write(addr, low);
        self.mem_write(addr + 1, high);
    }
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}
impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    fn load(&mut self, program: Vec<u8>) {
        // Reserve address 0x8000 to 0xFFFF for ROM. this.arr.splice(start, end, ...program);
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);

        //  0xFFFC is the reset vector. A memory location the processor reads,
        // when powered on or reset signal is received, to determine the address
        // from which to start executing code.
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;

        // Reset program_counter to the 2-byte value stored at 0xFFFC
        self.program_counter = self.mem_read_u16(0xFFFC)
    }

    fn run(&mut self) {
        let ref op_codes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let program_counter_state = self.program_counter;

            let op = op_codes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));

            match code {
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.lda(&op.mode),

                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.sta(&op.mode),

                0xAA => self.tax(),
                0xe8 => self.inx(),
                0x00 => return,
                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (op.len - 1) as u16;
            }
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode /*Immediate*/);
        let value = self.mem_read(addr);
        // vec![0xa5, 0x10, 0x00]
        self.register_a = value;

        /*
            Carry Flag -> Not affected
            Zero Flag -> Set if A = 0
            Interrupted Disable -> Not affected
            Decimal Mode Flag -> Not affected
            Break Command -> Not affected
            Overflow Flag -> Not affected
            Negative Flag -> Set if bit 7 of A is set
         */
        self.update_zero_and_negative_flags(self.register_a)
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x)
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;

                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;

                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);

                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);

                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x) as u16;

                let lo = self.mem_read(ptr) as u16;
                let hi = self.mem_read(ptr.wrapping_add(1) as u16) as u16;

                hi << 8 | lo
            }

            AddressingMode::NoneAddressing => panic!("Mode {:?} is not supported", mode),

            _ => todo!(),
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // result: 0x10 = 0b10000 = 16
        // status = (  0b0000_0000) = 0
        //          (& 0b1111_1101)
        self.status = if result == 0 {
            self.status | 0b0000_0010 // 2
        } else {
            self.status & 0b1111_1101 // 253
        };

        // result: 0x10 = 16 = (  0b0001_0000) = 0
        //                     (& 0b1000_0000)
        // status = (  0b0000_0000) = 0b0111_1111
        //          (| 0b0111_1111)
        self.status = if result & 0b1000_0000 != 0 {
            self.status | 0b1000_0000 // 128
        } else {
            self.status & 0b0111_1111 // 127
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        // register_a = 0x05
        // status = 0b0000_0010
        // program_counter = 2
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        // status = 0b0000_0010 == 2
        assert!(cpu.status & 0b0000_0010 == 0b10)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0A, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }
}

// NES implements typical von Neumann architecture
