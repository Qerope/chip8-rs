use std::fs::File;
use std::io::Read;
use rand::random;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: Vec<u16>,
    pub gfx: [bool; WIDTH * HEIGHT],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keys: [bool; 16],
    waiting_key: Option<usize>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut memory = [0u8; 4096];
        memory[..80].copy_from_slice(&FONT);
        Chip8 {
            memory,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: Vec::new(),
            gfx: [false; WIDTH * HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; 16],
            waiting_key: None,
        }
    }

    pub fn load_rom(&mut self, path: &str) -> std::io::Result<()> {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        self.memory[0x200..0x200 + buf.len()].copy_from_slice(&buf);
        Ok(())
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn cycle(&mut self) {
        let op = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[self.pc as usize + 1] as u16;
        self.pc += 2;
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        let x = ((op >> 8) & 0xF) as usize;
        let y = ((op >> 4) & 0xF) as usize;
        let n = (op & 0xF) as u8;
        let nn = (op & 0xFF) as u8;
        let nnn = op & 0xFFF;

        match op >> 12 {
            0x0 => match op {
                0x00E0 => {
                    self.gfx = [false; WIDTH * HEIGHT];
                }
                0x00EE => self.pc = self.stack.pop().expect("stack underflow"),
                _ => {}
            },
            0x1 => self.pc = nnn,
            0x2 => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            0x3 => if self.v[x] == nn { self.pc += 2 },
            0x4 => if self.v[x] != nn { self.pc += 2 },
            0x5 => if self.v[x] == self.v[y] { self.pc += 2 },
            0x6 => self.v[x] = nn,
            0x7 => self.v[x] = self.v[x].wrapping_add(nn),
            0x8 => match n {
                0x0 => self.v[x] = self.v[y],
                0x1 => self.v[x] |= self.v[y],
                0x2 => self.v[x] &= self.v[y],
                0x3 => self.v[x] ^= self.v[y],
                0x4 => {
                    let (r, c) = self.v[x].overflowing_add(self.v[y]);
                    self.v[0xF] = c as u8;
                    self.v[x] = r;
                }
                0x5 => {
                    let (r, b) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[0xF] = !b as u8;
                    self.v[x] = r;
                }
                0x6 => {
                    self.v[x] >>= 1;
                    self.v[0xF] = self.v[x] & 1;
                }
                0x7 => {
                    let (r, b) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[0xF] = !b as u8;
                    self.v[x] = r;
                }
                0xE => {
                    self.v[x] <<= 1;
                    self.v[0xF] = self.v[x] >> 7;
                }
                _ => {}
            },
            0x9 => if self.v[x] != self.v[y] { self.pc += 2 },
            0xA => self.i = nnn,
            0xB => self.pc = nnn + self.v[0] as u16,
            0xC => self.v[x] = random::<u8>() & nn,
            0xD => {
                let vx = self.v[x] as usize;
                let vy = self.v[y] as usize;
                self.v[0xF] = 0;
                for row in 0..n as usize {
                    let byte = self.memory[self.i as usize + row];
                    for col in 0..8 {
                        if byte & (0x80 >> col) == 0 {
                            continue;
                        }
                        let idx = (vy + row) * WIDTH + vx + col;
                        if self.gfx[idx] {
                            self.v[0xF] = 1;
                        }
                        self.gfx[idx] ^= true;
                    }
                }
            }
            0xE => match nn {
                0x9E => if self.keys[self.v[x] as usize & 0xF] { self.pc += 2 },
                0xA1 => if !self.keys[self.v[x] as usize & 0xF] { self.pc += 2 },
                _ => {}
            },
            0xF => match nn {
                0x07 => self.v[x] = self.delay_timer,
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x0A => match self.keys.iter().position(|&k| k) {
                    Some(k) => self.v[x] = k as u8,
                    None => self.pc -= 2,
                },
                0x1E => self.i += self.v[x] as u16,
                0x29 => self.i = (self.v[x] & 0xF) as u16 * 5,
                0x33 => {
                    let i = self.i as usize;
                    self.memory[i] = self.v[x] / 100;
                    self.memory[i + 1] = self.v[x] / 10 % 10;
                    self.memory[i + 2] = self.v[x] % 10;
                }
                0x55 => {
                    for k in 0..=x {
                        self.memory[self.i as usize + k] = self.v[k];
                    }
                    self.i += x as u16 + 1;
                }
                0x65 => {
                    for k in 0..=x {
                        self.v[k] = self.memory[self.i as usize + k];
                    }
                    self.i += x as u16 + 1;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
