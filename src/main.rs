use std::time::Duration;

use bit_vec::BitVec;
use chan::tick;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Processor {
    instruction: u16,
    program_counter: u16,
    index: u16,
    delay_timer: u16,
    sound_timer: u16,
    stack: Vec<u16>,
    memory: Vec<u8>,
    registers: Vec<u8>,
    display: BitVec,
    tick_rate: u64,
}

impl Processor {
    pub fn new() -> Processor {
        let mut processor = Processor {
            instruction: 0x0,
            program_counter: 0x50,
            index: 0x0,
            delay_timer: 0x0,
            sound_timer: 0x0,
            stack: Vec::new(),
            memory: vec![0; 0x1000],
            registers: vec![0, 0xF],
            display: BitVec::from_elem(0x40 * 0x20, false),
            tick_rate: 2, // default 700
        };

        processor.load_font();

        processor
    }

    pub fn main_loop(&mut self) -> Result<(), String> {
        let timer = tick(Duration::from_nanos(1000000000 / self.tick_rate));

        loop {
            timer.recv();

            self.fetch()?;
            println!("Fetched: {:#06X}", self.instruction);
            self.execute()?;
        }
    }

    fn load_font(&mut self) {
        let memory = &mut self.memory;

        memory[0x50..0xA0].copy_from_slice(&FONT);
    }

    fn fetch(&mut self) -> Result<(), String> {
        let memory = &mut self.memory;

        if self.program_counter as usize > memory.len() - 2 {
            return Err("Program counter out of bounds!".to_string());
        }

        self.instruction = ((memory[self.program_counter as usize] as u16) << 8)
            | (memory[(self.program_counter + 1) as usize] as u16);

        self.program_counter += 2;

        Ok(())
    }

    fn execute(&mut self) -> Result<(), String> {
        let stack = &mut self.stack;

        let instruction = self.instruction;

        let nibbles = (
            (instruction & 0xF000) >> 12 as u8,
            (instruction & 0x0F00) >> 8 as u8,
            (instruction & 0x00F0) >> 4 as u8,
            (instruction & 0x000F) as u8,
        );

        let x = nibbles.1; // high
        let y = nibbles.2; // low
        let n = nibbles.3; // nibble
        let kk = instruction & 0x00FF; // byte
        let nnn = instruction & 0x0FFF; // addr

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => {} // CLS
            (0x0, 0x0, 0xE, 0xE) => {} // RET
            (0x0, _, _, _) => {}       // SYS addr
            (0x1, _, _, _) => {}       // JP addr
            (0x2, _, _, _) => {}       // CALL addr
            (0x3, _, _, _) => {}       // SE Vx, byte
            (0x4, _, _, _) => {}       // SNE Vx, byte
            (0x5, _, _, 0x0) => {}     // SE Vx, Vy
            (0x6, _, _, _) => {}       // LD Vx, byte
            (0x7, _, _, _) => {}       // ADD Vx, byte
            (0x8, _, _, 0x0) => {}     // LD Vx, Vy
            (0x8, _, _, 0x1) => {}     // OR Vx, Vy
            (0x8, _, _, 0x2) => {}     // AND Vx, Vy
            (0x8, _, _, 0x3) => {}     // XOR Vx, Vy
            (0x8, _, _, 0x4) => {}     // ADD Vx, Vy
            (0x8, _, _, 0x5) => {}     // SUB Vx, Vy
            (0x8, _, _, 0x6) => {}     // SHR Vx {, Vy}
            (0x8, _, _, 0x7) => {}     // SUBN Vx, Vy
            (0x8, _, _, 0xE) => {}     // SHL Vx {, Vy}
            (0x9, _, _, 0x0) => {}     // SNE Vx, Vy
            (0xA, _, _, _) => {}       // LD I, addr
            (0xB, _, _, _) => {}       // JP V0, addr
            (0xC, _, _, _) => {}       // RND Vx, byte
            (0xD, _, _, _) => {}       // DRW Vx, Vy, nibble
            (0xE, _, 0x9, 0xE) => {}   // SKP Vx
            (0xE, _, 0xA, 0x1) => {}   // SKNP Vx
            (0xF, _, 0x0, 0x7) => {}   // LD Vx, DT
            (0xF, _, 0x0, 0xA) => {}   // LD Vx, K
            (0xF, _, 0x1, 0x5) => {}   // LD DT, Vx
            (0xF, _, 0x1, 0x8) => {}   // LD ST, Vx
            (0xF, _, 0x1, 0xE) => {}   // ADD I, Vx
            (0xF, _, 0x2, 0x9) => {}   // LD F, Vx
            (0xF, _, 0x3, 0x3) => {}   // LD B, Vx
            (0xF, _, 0x5, 0x5) => {}   // LD [I], Vx
            (0xF, _, 0x6, 0x5) => {}   // LD Vx, [I]
            _ => return Err(format!("Invalid instruction: {:#06X}!", instruction)),
        }

        Ok(())
    }
}

fn main() {
    let mut processor = Processor::new();

    println!("{}", processor.main_loop().err().unwrap());
}