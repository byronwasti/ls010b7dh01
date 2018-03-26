#![no_std]
extern crate embedded_hal as hal;
mod reverse_bits;

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
    buffer: [[u8; 128]; 2],
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

        let buffer = [[0; 128]; 2];

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

    pub fn write_data(&mut self) {
        for i in 0..2 {
            for j in 0..64 {
                self.buffer[i][j*2 + i] = 1;
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
            for j in 0..16 {
                let vals = &self.buffer[(i%2) as usize][j*8..(j*8)+8];
                let out = vals.iter()
                    .enumerate()
                    .map(|(idx, val)| val << idx)
                    .fold(0, |acc, x| acc | x);
                buffer[j+1] = out;
            }

            self.spi.write(&buffer);
        }

        // Write our final ending
        self.spi.write(&[0x00]);

        self.cs.set_low();
    }

    pub fn clear(&mut self) {
        self.write_spi(&[0x20, 0x00]);
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

