pub struct CPU {
    pub register_a: u8,
    pub status: u8,
    pub program_counter: u8,
}
impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let ops_code = program[self.program_counter as usize];
            self.program_counter += 1;

            match ops_code {
                0xA9 => {
                    let param = program[self.program_counter as usize];
                    self.register_a = param;

                    self.program_counter += 1;

                    if self.status == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    // status = 0b0000_0010
                    // register_a = 0x05 & 0b1000_0000 == 0
                    if self.register_a & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }
                0x00 => {
                    return;
                }
                _ => todo!(),
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.status & 0b0000_0010 == 0b10);
    }
}
