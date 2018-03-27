#![no_std]
extern crate embedded_hal as hal;
mod reverse_bits;
mod buffer_position;

use hal::blocking::spi::Write;
use hal::spi::{Mode, Phase, Polarity};
use hal::digital::OutputPin;


pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnSecondTransition,
};

pub struct Ls010b7dh01<SPI, CS, DISP> {
    spi: SPI,
    cs: CS,
    disp: DISP,
    buffer: [[u8; 16]; 128],
}

impl<SPI, CS, DISP, E> Ls010b7dh01<SPI, CS, DISP>
where
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    DISP: OutputPin,
{
    pub fn new(spi: SPI, mut cs: CS, mut disp: DISP) -> Self {
        disp.set_low();
        cs.set_low();

        let buffer = [[0; 16]; 128];

        Self {
            spi,
            cs,
            disp,
            buffer,
        }
    }

    pub fn enable(&mut self) {
        self.disp.set_high();
    }

    pub fn disable(&mut self) {
        self.disp.set_low();
    }

    pub fn write_pixel(&mut self, x: u8, y: u8, val: bool) {
        if x > 128 || y > 128 {
            return
        }

        let (bit, bucket) = buffer_position::get_position(x);

        if val {
            self.buffer[y as usize][bucket as usize] &= !(1 << bit);
        } else {
            self.buffer[y as usize][bucket as usize] |= 1 << bit;
        }
    }

    pub fn write_checkerboard(&mut self) {
        for i in 0..128 {
            for j in 0..64 {
                self.write_pixel(j*2 + i%2, i, true);
            }
        }
    }

    /// X/Y are bottom left of rect
    pub fn draw_rect(&mut self, x: u8, y: u8, width: u8, height: u8) {
        if x > 128 || y > 128 {
            return
        }

        let x_end = x + width;
        let y_end = y + height;

        for i in x..x_end {
            if i > 128 {
                break; 
            }

            self.write_pixel(i, y, true);
            if y_end < 128 {
                self.write_pixel(i, y_end-1, true);
            }
        }

        for i in y..y_end {
            if i > 128 {
                break;
            }

            self.write_pixel(x, i, true);
            if x_end < 128 {
                self.write_pixel(x_end-1, i, true);
            }
        }
    }

    pub fn draw_circle(&mut self, x0: u8, y0: u8, r: u8) {
        let x0 = x0 as i32;
        let y0 = y0 as i32;
        let r = r as i32;

        let mut x: i32 = r - 1;
        let mut y: i32 = 0;
        let mut dx: i32 = 1;
        let mut dy: i32 = 1;
        let mut err: i32 = dx - (r << 1);

        while x >= y {
            self.write_pixel((x0 + x) as u8, (y0 + y) as u8, true);
            self.write_pixel((x0 + y) as u8, (y0 + x) as u8, true);
            self.write_pixel((x0 - y) as u8, (y0 + x) as u8, true);
            self.write_pixel((x0 - x) as u8, (y0 + y) as u8, true);

            self.write_pixel((x0 - x) as u8, (y0 - y) as u8, true);
            self.write_pixel((x0 - y) as u8, (y0 - x) as u8, true);
            self.write_pixel((x0 + y) as u8, (y0 - x) as u8, true);
            self.write_pixel((x0 + x) as u8, (y0 - y) as u8, true);

            if (err <= 0) {
                y += 1;
                err += dy;
                dy += 2;
            }

            if (err > 0) {
                x -= 1;
                dx += 2;
                err += dx - (r << 1);
            }
        }
    }

    pub fn write_dotted_line(&mut self) {
        self.write_spi(&[ 0x80, 0x82,
        0x33, 0x33, 0x33, 0x33,
        0x33, 0x33, 0x33, 0x33,
        0x33, 0x33, 0x33, 0x33,
        0x33, 0x33, 0x33, 0x33,
        0x00, 0x00 ]);
    }

    pub fn flush_buffer(&mut self) {
        self.cs.set_high();

        // Write main message
        self.spi.write(&[ 0x80 ]);

        // Pack buffer into byte form and send
        let mut buffer = [0; 18];
        for i in 0..128 {
            buffer[0] = reverse_bits::msb2lsb(i+1);
            buffer[1..17].clone_from_slice(&self.buffer[i as usize][0..16]);
            self.spi.write(&buffer);
        }

        // Write our final ending
        self.spi.write(&[0x00]);

        self.cs.set_low();
    }

    pub fn clear(&mut self) {
        self.write_spi(&[0x20, 0x00]);

        for line in self.buffer.iter_mut() {
            for elem in line {
                *elem = 0xFF;
            }
        }
    }

    pub fn display_mode(&mut self) {
        self.write_spi(&[0x00, 0x00]);
    }

    fn write_spi(&mut self, data: &[u8]) {
        self.cs.set_high();

        let _ = self.spi.write(data);

        self.cs.set_low();
    }
}

