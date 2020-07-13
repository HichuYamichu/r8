use rand::prelude::*;

const OPCODE_SIZE: usize = 2;
const PROGRAM_START: usize = 0x200;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct CPU {
    i: usize,
    pc: usize,
    mem: [u8; 4096],
    v: [u8; 16],
    stack: [usize; 16],
    sp: usize,
    dt: u8,
    st: u8,
    vmem: [[u8; WIDTH]; HEIGHT],
    vmem_changed: bool,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            i: 0,
            pc: PROGRAM_START,
            mem: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            vmem: [[0; WIDTH]; HEIGHT],
            vmem_changed: false,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        }
    }

    pub fn step(&mut self) {
        let op = self.opcode();
        self.exec(op);
    }

    fn opcode(&self) -> u16 {
        (self.mem[self.pc as usize] as u16) << 8 | (self.mem[(self.pc + 1) as usize] as u16)
    }

    fn exec(&mut self, op: u16) {
        let nibbles = (
            (op & 0xF000) >> 12 as u8,
            (op & 0x0F00) >> 8 as u8,
            (op & 0x00F0) >> 4 as u8,
            (op & 0x000F) as u8,
        );
        let nnn = (op & 0x0FFF) as usize;
        let nn = (op & 0x00FF) as u8;
        let n = nibbles.3 as usize;
        let y = nibbles.2 as usize;
        let x = nibbles.1 as usize;

        match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xnn(x, nn),
            (0x04, _, _, _) => self.op_4xnn(x, nn),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xnn(x, nn),
            (0x07, _, _, _) => self.op_7xnn(x, nn),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8x06(x),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0e) => self.op_8xye(x),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0a, _, _, _) => self.op_annn(nnn),
            (0x0b, _, _, _) => self.op_bnnn(nnn),
            (0x0c, _, _, _) => self.op_cxnn(x, nn),
            (0x0d, _, _, _) => self.op_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(x),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(x),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(x),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(x),
            _ => unreachable!(),
        };
    }

    fn op_00e0(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.vmem[y][x] = 0;
            }
        }
        self.vmem_changed = true;
        self.pc += OPCODE_SIZE;
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn op_1nnn(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: usize) {
        self.stack[self.sp] = (self.pc + OPCODE_SIZE).into();
        self.sp += 1;
        self.pc = nnn;
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2 * OPCODE_SIZE;
        } else {
            self.pc += OPCODE_SIZE;
        }
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2 * OPCODE_SIZE;
        } else {
            self.pc += OPCODE_SIZE;
        }
    }

    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2 * OPCODE_SIZE;
        } else {
            self.pc += OPCODE_SIZE;
        }
    }

    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
        self.pc += OPCODE_SIZE;
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) {
        let vx = self.v[x] as u16;
        let val = nn as u16;
        let result = vx + val;
        self.v[x] = result as u8;
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.pc += OPCODE_SIZE;
    }

    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
        self.pc += OPCODE_SIZE;
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
        self.pc += OPCODE_SIZE;
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
        self.pc += OPCODE_SIZE;
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        let xy = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = xy as u8;
        self.v[0x0f] = if xy > 0xFF { 1 } else { 0 };
        self.pc += OPCODE_SIZE;
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        self.v[0x0f] = if self.v[x] > self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        self.pc += OPCODE_SIZE;
    }

    fn op_8x06(&mut self, x: usize) {
        self.v[0x0f] = self.v[x] & 1;
        self.v[x] >>= 1;
        self.pc += OPCODE_SIZE;
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        self.v[0x0f] = if self.v[y] > self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        self.pc += OPCODE_SIZE;
    }

    fn op_8xye(&mut self, x: usize) {
        self.v[0x0f] = (self.v[x] & 0b10000000) >> 7;
        self.v[x] <<= 1;
        self.pc += OPCODE_SIZE;
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2 * OPCODE_SIZE;
        } else {
            self.pc += OPCODE_SIZE;
        }
    }

    fn op_annn(&mut self, nnn: usize) {
        self.i = nnn;
        self.pc += OPCODE_SIZE;
    }

    fn op_bnnn(&mut self, nnn: usize) {
        self.pc = nnn + (self.v[0] as usize);
    }

    fn op_cxnn(&mut self, x: usize, nn: u8) {
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & nn;
        self.pc += OPCODE_SIZE;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        self.v[0x0f] = 0;
        for byte in 0..n {
            let y = (self.v[y] as usize + byte) % HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % WIDTH;
                let color = (self.mem[self.i + byte] >> (7 - bit)) & 1;
                self.v[0x0f] |= color & self.vmem[y][x];
                self.vmem[y][x] ^= color;
            }
        }
        self.vmem_changed = true;
        self.pc += OPCODE_SIZE;
    }

    fn op_ex9e(&mut self, x: usize) {
        if self.keypad[self.v[x] as usize] {
            self.pc += 2 * OPCODE_SIZE;
        } else {
            self.pc += OPCODE_SIZE;
        }
    }

    fn op_exa1(&mut self, x: usize) {
        if !self.keypad[self.v[x] as usize] {
            self.pc += 2 * OPCODE_SIZE;
        } else {
            self.pc += OPCODE_SIZE;
        }
    }

    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.dt;
        self.pc += OPCODE_SIZE;
    }

    fn op_fx0a(&mut self, x: usize) {
        self.keypad_waiting = true;
        self.keypad_register = x;
        self.pc += OPCODE_SIZE;
    }

    fn op_fx15(&mut self, x: usize) {
        self.dt = self.v[x];
        self.pc += OPCODE_SIZE;
    }

    fn op_fx18(&mut self, x: usize) {
        self.st = self.v[x];
        self.pc += OPCODE_SIZE;
    }

    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as usize;
        self.v[0x0f] = if self.i > 0x0F00 { 1 } else { 0 };
        self.pc += OPCODE_SIZE;
    }

    fn op_fx29(&mut self, x: usize) {
        self.i = (self.v[x] as usize) * 5;
        self.pc += OPCODE_SIZE;
    }

    fn op_fx33(&mut self, x: usize) {
        self.mem[self.i] = self.v[x] / 100;
        self.mem[self.i + 1] = (self.v[x] % 100) / 10;
        self.mem[self.i + 2] = self.v[x] % 10;
        self.pc += OPCODE_SIZE;
    }

    fn op_fx55(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.mem[self.i + i] = self.v[i];
        }
        self.pc += OPCODE_SIZE;
    }

    fn op_fx65(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.v[i] = self.mem[self.i + i];
        }
        self.pc += OPCODE_SIZE;
    }
}
