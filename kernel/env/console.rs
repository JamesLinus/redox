use alloc::boxed::Box;

use collections::String;
use collections::Vec;

use common::event::{self, Event, EventOption};

use core::mem;

use drivers::io::{Io, Pio};

use graphics::color::Color;
use graphics::display::Display;

use sync::WaitQueue;

const BLACK: Color = Color::new(0, 0, 0);
const RED: Color = Color::new(194, 54, 33);
const GREEN: Color = Color::new(37, 188, 36);
const YELLOW: Color = Color::new(173, 173, 39);
const BLUE: Color = Color::new(73, 46, 225);
const MAGENTA: Color = Color::new(211, 56, 211);
const CYAN: Color = Color::new(51, 187, 200);
const WHITE: Color = Color::new(203, 204, 205);

pub struct Console {
    pub display: Option<Box<Display>>,
    pub point_x: usize,
    pub point_y: usize,
    pub foreground: Color,
    pub background: Color,
    pub draw: bool,
    pub redraw: bool,
    pub command: String,
    pub commands: WaitQueue<String>,
    pub escape: bool,
    pub escape_sequence: bool,
    pub sequence: Vec<String>,
}

impl Console {
    pub fn new() -> Console {
        Console {
            display: Display::root(),
            point_x: 0,
            point_y: 0,
            foreground: WHITE,
            background: BLACK,
            draw: false,
            redraw: true,
            command: String::new(),
            commands: WaitQueue::new(),
            escape: false,
            escape_sequence: false,
            sequence: Vec::new(),
        }
    }

    pub fn code(&mut self, c: char) {
        if self.escape_sequence {
            if c >= '0' && c <= '9' {
                // Add a number to the sequence list
                if let Some(mut value) = self.sequence.last_mut() {
                    value.push(c);
                }
            } else if c == ';' {
                // Split sequence into list
                self.sequence.push(String::new());
            } else if c == 'm' {
                // Display attributes
                for value in self.sequence.iter() {
                    if value == "0" {
                        // Reset all
                        self.foreground = WHITE;
                        self.background = BLACK;
                    } else if value == "30" {
                        self.foreground = BLACK;
                    } else if value == "31" {
                        self.foreground = RED;
                    } else if value == "32" {
                        self.foreground = GREEN;
                    } else if value == "33" {
                        self.foreground = YELLOW;
                    } else if value == "34" {
                        self.foreground = BLUE;
                    } else if value == "35" {
                        self.foreground = MAGENTA;
                    } else if value == "36" {
                        self.foreground = CYAN;
                    } else if value == "37" {
                        self.foreground = WHITE;
                    } else if value == "40" {
                        self.background = BLACK;
                    } else if value == "41" {
                        self.background = RED;
                    } else if value == "42" {
                        self.background = GREEN;
                    } else if value == "43" {
                        self.background = YELLOW;
                    } else if value == "44" {
                        self.background = BLUE;
                    } else if value == "45" {
                        self.background = MAGENTA;
                    } else if value == "46" {
                        self.background = CYAN;
                    } else if value == "47" {
                        self.background = WHITE;
                    }
                }

                self.escape_sequence = false;
            } else if c == 'H' || c == 'f' {
                if let Some(ref mut display) = self.display {
                    display.rect(self.point_x, self.point_y, 8, 16, self.background);
                }

                let row = self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(0);
                self.point_y = row * 16;

                let col = self.sequence.get(1).map_or("", |p| &p).parse::<usize>().unwrap_or(0);
                self.point_x = col * 8;

                if let Some(ref mut display) = self.display {
                    display.rect(self.point_x, self.point_y, 8, 16, self.foreground);
                }
                self.redraw = true;

                self.escape_sequence = false;
            } else {
                self.escape_sequence = false;
            }

            if !self.escape_sequence {
                self.sequence.clear();
                self.escape = false;
            }
        } else if c == '[' {
            // Control sequence initiator

            self.escape_sequence = true;
            self.sequence.push(String::new());
        } else if c == 'c' {
            // Reset
            self.point_x = 0;
            self.point_y = 0;
            self.foreground = WHITE;
            self.background = BLACK;
            if let Some(ref mut display) = self.display {
                display.set(self.background);
            }
            self.redraw = true;

            self.escape = false;
        } else {
            // Unknown escape character

            self.escape = false;
        }
    }

    pub fn character(&mut self, c: char) {
        if let Some(ref mut display) = self.display {
            display.rect(self.point_x, self.point_y, 8, 16, self.background);
            if c == '\x00' {
                // Ignore null character
            } else if c == '\x1B' {
                self.escape = true;
            } else if c == '\n' {
                self.point_x = 0;
                self.point_y += 16;
            } else if c == '\t' {
                self.point_x = ((self.point_x / 64) + 1) * 64;
            } else if c == '\x08' {
                if self.point_x >= 8 {
                    self.point_x -= 8;
                }
                display.rect(self.point_x, self.point_y, 8, 16, self.background);
            } else {
                display.char(self.point_x, self.point_y, c, self.foreground);
                self.point_x += 8;
            }
            if self.point_x >= display.width {
                self.point_x = 0;
                self.point_y += 16;
            }
            while self.point_y + 16 > display.height {
                display.scroll(16);
                self.point_y -= 16;
            }
            display.rect(self.point_x, self.point_y, 8, 16, self.foreground);
            self.redraw = true;
        }
    }

    pub fn event(&mut self, event: Event) {
        match event.to_option() {
            EventOption::Key(key_event) => {
                if key_event.pressed {
                    match key_event.scancode {
                        event::K_BKSP => if ! self.command.is_empty() {
                            self.write(&[8]);
                            self.command.pop();
                        },
                        _ => match key_event.character {
                            '\0' => (),
                            c => {
                                self.command.push(c);
                                self.write(&[c as u8]);

                                if c == '\n' {
                                    let mut command = String::new();
                                    mem::swap(&mut self.command, &mut command);
                                    self.commands.send(command);
                                }
                            }
                        },
                    }
                }
            }
            _ => (),
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        let serial_status = Pio::<u8>::new(0x3F8 + 5);
        let mut serial_data = Pio::<u8>::new(0x3F8);

        for byte in bytes.iter() {
            let c = *byte as char;

            if self.escape {
                self.code(c);
            } else {
                self.character(c);
            }

            while !serial_status.readf(0x20) {}
            serial_data.write(*byte);

            if *byte == 8 {
                while !serial_status.readf(0x20) {}
                serial_data.write(0x20);

                while !serial_status.readf(0x20) {}
                serial_data.write(8);
            }
        }

        if self.draw && self.redraw {
            self.redraw = false;
            if let Some(ref mut display) = self.display {
                display.flip();
            }
        }
    }
}
