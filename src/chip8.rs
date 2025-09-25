use rand;
use rand::RngCore;
use rand::{Rng, random};
use std::fs;

pub const DISP_WIDTH: usize = 64;
pub const DISP_HEIGHT: usize = 32;

// CHIP-8 built in fonts
// used by DXYN draw function in user programs.
// Hex digits 0-9 and A-F, 5 bytes each
const FONTS: [u8; 80] = [
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
const START_ADDR: usize = 0x200;

pub struct Chip8 {
    /// 4 kB of RAM memory;
    /// Addresses from 0x000 to 0xFF;
    /// 0x000 - 0x200 Interpreter,
    /// 0x200 - 0x600 User programs,
    /// 0x600 - 0xFFF ETI 660 User programs,
    pub ram: [u8; 4096],

    /// 16 8-bit registers V0,V1...VF
    pub registers: [u8; 16],

    /// Stack with size of 16
    pub stack: [u16; 16],

    /// display 64x32
    pub video: [bool; DISP_WIDTH * DISP_HEIGHT],
    pub keypad: [u8; 16],

    /// SP stack pointer
    pub sp: u8,

    /// PC program counter
    pub pc: u16,

    /// I index register
    pub i: u16,

    /// ST sound timer
    pub st: u8,

    /// DT delay time
    pub dt: u8,

    /// Code of current operation
    pub opcode: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip = Chip8 {
            ram: [0; 4096],
            registers: [0; 16],
            stack: [0; 16],
            keypad: [0; 16],
            video: [false; DISP_WIDTH * DISP_HEIGHT],
            sp: 0,
            pc: 0x200,
            i: 0,
            st: 0,
            dt: 0,
            opcode: 0,
        };

        // Load fonts into memory
        chip.ram[..FONTS.len()].copy_from_slice(&FONTS);

        chip
    }

    pub fn load_rom(&mut self, fname: &str) {
        let contents = fs::read(fname).expect("Should have been able to read the file");
        let mut idx = START_ADDR;
        for content in contents {
            if idx > 0xFFF {
                panic!("Not supported address {:#x}", idx)
            }
            self.ram[idx] = content;
            idx += 1;
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn set_key_value(&mut self, key: usize, value: u8) {
        self.keypad[key] = value;
    }

    pub fn cycle(&mut self) {
        if self.pc > 0xFFF {
            panic!("Not supported address {:#x}", self.pc)
        }
        let opcode: u16 =
            (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;

        self.opcode = opcode;
        self.pc += 2;

        let digit_1 = (opcode & 0xF000) >> 12;
        let digit_2 = (opcode & 0x0F00) >> 8;
        let digit_3 = (opcode & 0x00F0) >> 4;
        let digit_4 = (opcode & 0x000F);

        match (digit_1, digit_2, digit_3, digit_4) {
            (0, 0, 0xE, 0) => self.op_00e0(),
            (0, 0, 0xE, 0xE) => self.op_00ee(),
            (1, _, _, _) => self.op_1nnn(),
            (2, _, _, _) => self.op_2nnn(),
            (3, _, _, _) => self.op_3xnn(),
            (4, _, _, _) => self.op_4xnn(),
            (5, _, _, 0) => self.op_5xy0(),
            (6, _, _, _) => self.op_6xnn(),
            (7, _, _, _) => self.op_7xnn(),
            (8, _, _, 0) => self.op_8xy0(),
            (8, _, _, 1) => self.op_8xy1(),
            (8, _, _, 2) => self.op_8xy2(),
            (8, _, _, 3) => self.op_8xy3(),
            (8, _, _, 4) => self.op_8xy4(),
            (8, _, _, 5) => self.op_8xy5(),
            (8, _, _, 6) => self.op_8xy6(),
            (8, _, _, 7) => self.op_8xy7(),
            (8, _, _, 0xE) => self.op_8xye(),
            (9, _, _, 0) => self.op_9xy0(),
            (0xA, _, _, _) => self.op_annn(),
            (0xB, _, _, _) => self.op_bnnn(),
            (0xC, _, _, _) => self.op_cxnn(),
            (0xD, _, _, _) => self.op_dxyn(),
            (0xE, _, 9, 0xE) => self.op_ex9e(),
            (0xE, _, 0xA, 1) => self.op_exa1(),
            (0xF, _, 0, 7) => self.op_fx07(),
            (0xF, _, 0, 0xA) => self.op_fx0a(),
            (0xF, _, 1, 5) => self.op_fx15(),
            (0xF, _, 1, 8) => self.op_fx18(),
            (0xF, _, 1, 0xE) => self.op_fx1e(),
            (0xF, _, 2, 9) => self.op_fx29(),
            (0xF, _, 3, 3) => self.op_fx33(),
            (0xF, _, 5, 5) => self.op_fx55(),
            (0xF, _, 6, 5) => self.op_fx65(),
            _ => {
                panic!("Illegal OP {:#x}", opcode);
            }
        };
    }

    /// `CLS`
    /// Clear display
    fn op_00e0(&mut self) {
        self.video = [false; DISP_WIDTH * DISP_HEIGHT];
    }

    /// `RET`
    /// Return from subroutine.
    /// Pop address from stack and set PC to popped address.
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    /// `JMP NNN`
    /// Jump to address NNN.
    /// PC = NNN
    fn op_1nnn(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }

    /// `CALL NNN`
    /// Put current PC on stack
    /// PC = NNN
    fn op_2nnn(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    /// `SE Vx, NN`
    /// Skip next operation if Vx == NN.
    fn op_3xnn(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let nn: u8 = (self.opcode & 0x00FF) as u8;
        if self.registers[x] == nn {
            self.pc += 2;
        }
    }

    /// `SNE Vx, NN`
    /// Skip next operation if Vx != NN.
    fn op_4xnn(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let nn: u8 = (self.opcode & 0x00FF) as u8;
        if self.registers[x] != nn {
            self.pc += 2;
        }
    }

    /// `SE Vx, VY`
    /// Skip next operation if Vx == Vy.
    fn op_5xy0(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] == self.registers[y] {
            self.pc += 2;
        }
    }

    /// `LD Vx, NN`
    /// Vx = NN
    fn op_6xnn(&mut self) {
        let x: usize = (self.opcode as usize & 0x0F00) >> 8;
        let nn: u8 = (self.opcode & 0xFF) as u8;
        self.registers[x] = nn;
    }

    /// `ADD Vx, NN`
    /// Vx = Vx + NN
    fn op_7xnn(&mut self) {
        let x: usize = (self.opcode as usize & 0x0F00) >> 8;
        let nn: u8 = (self.opcode & 0xFF) as u8;
        self.registers[x] = self.registers[x].wrapping_add(nn);
    }

    /// `LD Vx, Vy`
    /// Vy = Vx
    fn op_8xy0(&mut self) {
        let v_x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let v_y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[v_x] = self.registers[v_y];
    }

    /// `OR Vx, Vy`
    /// Vx = Vx | Vy
    fn op_8xy1(&mut self) {
        let v_x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let v_y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[v_x] |= self.registers[v_y];
    }

    /// `AND Vx, Vy`
    /// Vx = Vx & Vy
    fn op_8xy2(&mut self) {
        let v_x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let v_y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[v_x] &= self.registers[v_y];
    }

    /// `XOR Vx, Vy`
    /// Vx = Vx ^ Vy
    fn op_8xy3(&mut self) {
        let v_x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let v_y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        self.registers[v_x] ^= self.registers[v_y];
    }

    /// `ADD Vx, Vy`
    /// Vx = Vx + Vy
    fn op_8xy4(&mut self) {
        let v_x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let v_y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        let (new_vx, carry) = self.registers[v_x].overflowing_add(self.registers[v_y]);
        self.registers[0xF] = if carry { 1 } else { 0 };
        self.registers[v_x] = new_vx;
    }

    /// `SUB Vx, Vy`
    /// Vx = Vx - Vy
    fn op_8xy5(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        let (new_vx, borrow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[0xF] = if borrow { 0 } else { 1 };
        self.registers[x] = new_vx;
    }

    /// `SHR Vx, Vy`
    /// Vx = Vx >> 1
    /// Vy ignored
    fn op_8xy6(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[0xF] = self.registers[x] & 1;
        self.registers[x] >>= 1;
    }

    /// `SUBN Vy, Vx`
    /// Vx to Vy - Vx
    /// if underflows VF = 0, else VF = 1
    fn op_8xy7(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        let (new_vx, borrow) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[0xF] = if borrow { 0 } else { 1 };
        self.registers[x] = new_vx;
    }

    /// `SHL Vx, VY`
    /// VF = most significant bit of Vx;
    /// Vx = Vx << 1
    fn op_8xye(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;

        self.registers[0xF] = self.registers[x] & (0x1 << 7);
        self.registers[x] <<= 1;
    }

    /// `SNE Vx, Vy`
    /// Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }

    /// `LD I, NNN`
    /// I = NNN
    fn op_annn(&mut self) {
        self.i = self.opcode & 0x0FFF;
    }

    /// `JMP V0, NNN`
    /// PC = V0 + NNN
    fn op_bnnn(&mut self) {
        let nnn = self.opcode & 0xFFF;
        self.pc = (self.registers[0] as u16) + nnn;
    }

    /// `RND Vx, NN`
    /// Vx = random byte & NN
    fn op_cxnn(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let nn: u8 = (self.opcode & 0x00FF) as u8;
        let rng: u8 = random();
        self.registers[x] = nn & rng;
    }

    /// `DRW Vx, Vy, N`
    /// Draw N-byte sized sprite from `RAM[I]` to display at `[Vx][Vy]`.
    fn op_dxyn(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        let sprite_length: u8 = (self.opcode & 0x000F) as u8;

        let x_coord = self.registers[x] % DISP_WIDTH as u8;
        let y_coord = self.registers[y] % DISP_WIDTH as u8;

        let mut collision = false;

        // Draw sprite byte after byte, bottom up
        for row in 0..sprite_length {
            // Load another byte of sprite data from RAM at I
            let ram_idx: usize = (self.i + row as u16) as usize;
            let sprite_byte = self.ram[ram_idx];

            // Current y coord of sprite
            let curr_y = (y_coord + row) as usize % DISP_HEIGHT;

            // Draw all bits in row
            for col in 0..8 {
                // Current x coord of sprite
                let curr_x = (x_coord + col) as usize % DISP_WIDTH;
                let idx = curr_y * DISP_WIDTH + curr_x;

                // Get another sprite bit and draw it
                let sprite_bit = sprite_byte & (0x1 << 7 - col);
                if sprite_bit > 0 {
                    // Collision -> bit of sprite is already set on display
                    if self.video[idx] {
                        collision = true;
                    }
                    self.video[idx] ^= true;
                }
            }
        }

        self.registers[0xF] = if collision { 1 } else { 0 };
    }

    /// `SKP Vx`
    /// Skip next instruction if key with value of Vx is pressed
    fn op_ex9e(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.registers[x] as usize] > 0 {
            self.pc += 2;
        }
    }

    /// `SKNP Vx`
    /// Skip next instruction if key with value of Vx is not pressed
    fn op_exa1(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.registers[x] as usize] == 0 {
            self.pc += 2;
        }
    }

    /// `LD Vx, DT`
    /// DT = Vx (set delay timer to Vx)
    fn op_fx07(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[x] = self.dt;
    }

    /// `LD Vx, KEY`
    /// Wait for KEY press and store KEY value in Vx
    fn op_fx0a(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;

        if self.keypad[x] > 0 {
            self.registers[x] = x as u8;
        } else {
            self.pc -= 2;
        }
    }

    /// `LD DT, Vx`
    /// DT = Vx (set delay timer to Vx)
    fn op_fx15(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.dt = self.registers[x];
    }

    /// `LD ST, Vx`
    /// ST = Vx (set sount timer to Vx)
    fn op_fx18(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.st = self.registers[x];
    }

    /// `ADD I, Vx`
    /// Vx = Vx + I
    fn op_fx1e(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.i = self.i.wrapping_add(self.registers[x] as u16);
    }

    /// `LD I, FONT(Vx)`
    /// Load 5-byte Font character representing Vx to I.
    fn op_fx29(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.i = self.registers[x] as u16 * 5;
    }

    /// `BCD Vx`
    /// Decode Vx to decimal.
    /// Set `RAM[I], RAM[I+1], RAM[I+2]` to hundreds, tens and ones.
    /// 
    /// ex. for `Vx = 123` => `RAM[I] = 1; RAM[I+1] = 2; RAM[I+2] = 3`
    fn op_fx33(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let v_x: u8 = self.registers[x];

        let ones: u8 = v_x % 10;
        let tens: u8 = (v_x % 100 - ones) / 10;
        let hundreds: u8 = (v_x - (tens * 10) - ones) / 100;

        self.ram[self.i as usize] = hundreds;
        self.ram[(self.i + 1) as usize] = tens;
        self.ram[(self.i + 2) as usize] = ones;
    }

    /// `LD [I], VX`
    /// Load values of registers from V0 to Vx to memory starting at address I.
    fn op_fx55(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.ram[(self.i + i as u16) as usize] = self.registers[i];
        }
    }

    /// `LD VX, [I]`
    /// Load values from memory starting at address I to registers from V0 to Vx.
    fn op_fx65(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.registers[i] = self.ram[self.i as usize + i];
        }
        self.i = self.i.wrapping_add((x + 1) as u16);
    }
}
