use rand::prelude::*;
use sdl2::keyboard::Keycode;

use crate::HEIGHT;
use crate::WIDTH;

const OPCODE_SIZE: usize = 2;
const PROGRAM_START: usize = 0x200;
const FONTSET: [u8; 80] = [
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

pub struct CPU {
    i: usize,
    pc: usize,
    mem: [u8; 4096],
    v: [u8; 16],
    stack: [usize; 16],
    sp: usize,
    dt: u8,
    st: u8,
    pub display: [[u8; WIDTH]; HEIGHT],
    pub redraw: bool,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = Self {
            i: 0,
            pc: PROGRAM_START,
            mem: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            display: [[0; WIDTH]; HEIGHT],
            redraw: false,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        };

        for i in 0..80 {
            cpu.mem[i] = FONTSET[i];
        }

        cpu
    }

    pub fn key_down(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Num1 => self.keypad[0x1] = true,
            Keycode::Num2 => self.keypad[0x2] = true,
            Keycode::Num3 => self.keypad[0x3] = true,
            Keycode::Num4 => self.keypad[0xC] = true,
            Keycode::Q => self.keypad[0x4] = true,
            Keycode::W => self.keypad[0x5] = true,
            Keycode::E => self.keypad[0x6] = true,
            Keycode::R => self.keypad[0xD] = true,
            Keycode::A => self.keypad[0x7] = true,
            Keycode::S => self.keypad[0x8] = true,
            Keycode::D => self.keypad[0x9] = true,
            Keycode::F => self.keypad[0xE] = true,
            Keycode::Z => self.keypad[0xA] = true,
            Keycode::X => self.keypad[0x0] = true,
            Keycode::C => self.keypad[0xB] = true,
            Keycode::V => self.keypad[0xF] = true,
            _ => {}
        }
    }

    pub fn key_up(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Num1 => self.keypad[0x1] = false,
            Keycode::Num2 => self.keypad[0x2] = false,
            Keycode::Num3 => self.keypad[0x3] = false,
            Keycode::Num4 => self.keypad[0xC] = false,
            Keycode::Q => self.keypad[0x4] = false,
            Keycode::W => self.keypad[0x5] = false,
            Keycode::E => self.keypad[0x6] = false,
            Keycode::R => self.keypad[0xD] = false,
            Keycode::A => self.keypad[0x7] = false,
            Keycode::S => self.keypad[0x8] = false,
            Keycode::D => self.keypad[0x9] = false,
            Keycode::F => self.keypad[0xE] = false,
            Keycode::Z => self.keypad[0xA] = false,
            Keycode::X => self.keypad[0x0] = false,
            Keycode::C => self.keypad[0xB] = false,
            Keycode::V => self.keypad[0xF] = false,
            _ => {}
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = PROGRAM_START + i;
            if addr < 4096 {
                self.mem[addr] = byte;
            } else {
                break;
            }
        }
    }

    pub fn step(&mut self) {
        self.redraw = false;
        if self.keypad_waiting {
            for i in 0..self.keypad.len() {
                if self.keypad[i] {
                    self.keypad_waiting = false;
                    self.v[self.keypad_register] = i as u8;
                    break;
                }
            }
        } else {
            if self.dt > 0 {
                self.dt -= 1
            }
            if self.st > 0 {
                self.st -= 1
            }

            let op = self.opcode();
            self.exec(op);
        }
    }

    fn opcode(&self) -> u16 {
        (self.mem[self.pc] as u16) << 8 | (self.mem[self.pc + 1] as u16)
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
                self.display[y][x] = 0;
            }
        }
        self.redraw = true;
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
        self.pc += OPCODE_SIZE;
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
                self.v[0x0f] |= color & self.display[y][x];
                self.display[y][x] ^= color;
            }
        }

        self.redraw = true;
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
