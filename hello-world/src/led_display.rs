/*
    Created with reference to LedDisplay controller library for Avago HCMS-297x displays

    Original found at https://www.pjrc.com/teensy/td_libs_LedDisplay.html

    Controls an Avago HCMS29xx or HCMS39xx display. This display has 4-8 characters, each 5x7 LEDs
*/

#![cfg_attr(docsrs, feature(doc_cfg))]
//#![feature(alloc)]

// Need to reference this so that it doesn't get stripped out
#[cfg(target_arch = "arm")]
extern crate teensy4_fcb;
//extern crate alloc;

//use alloc::string::String;
use embedded_hal::digital::v2::{OutputPin};
pub use teensy4_pins::common;
pub use teensy4_pins::t40;
pub use teensy4_pins::t41;
use teensy4_bsp as bsp;
use log::{info};

pub use hal::Peripherals;
pub use imxrt_hal as hal;
pub use hal::gpio::GPIO;

// pub type dataPin = GPIO<t40::P6, hal::gpio::Output>;
// pub type registerSelectPin = GPIO<t40::P7, hal::gpio::Output>;
// pub type clockPin = GPIO<t40::P8, hal::gpio::Output>;
// pub type enablePin = GPIO<t40::P9, hal::gpio::Output>;
// pub type resetPin = GPIO<t40::P10, hal::gpio::Output>;

const LEDDISPLAY_MAXCHARS: u8 = 32;

pub struct Hcms <DataPin: OutputPin, RegisterSelectPin: OutputPin, ClockPin: OutputPin, EnablePin: OutputPin, ResetPin: OutputPin>
{
    data_pin: DataPin,
    register_select: RegisterSelectPin,
    clock_pin: ClockPin,
    enable_pin: EnablePin,
    reset_pin: ResetPin,
    display_length: u8,
    string_buffer: [u8; (LEDDISPLAY_MAXCHARS + 5) as usize],
    display_string: [u8; (LEDDISPLAY_MAXCHARS + 5) as usize],
    //display_string: String,
    cursor_pos: i8,
    dot_register: [u8; (LEDDISPLAY_MAXCHARS * 5) as usize ],
}

impl <DataPin: OutputPin, RegisterSelectPin: OutputPin, ClockPin: OutputPin, EnablePin: OutputPin, ResetPin: OutputPin>
Hcms<DataPin, RegisterSelectPin, ClockPin, EnablePin, ResetPin>
{
    pub fn new_display(
        data: DataPin,
        rs: RegisterSelectPin,
        clock: ClockPin,
        en: EnablePin,
        reset: ResetPin,
        displaylen: u8,
        //delay: &mut D,
    ) -> Self
    {
        let mut display = Hcms{
            data_pin: data,
            register_select: rs,
            clock_pin: clock,
            enable_pin: en,
            reset_pin: reset,
            display_length: displaylen,
            string_buffer: [b' ' as u8; (LEDDISPLAY_MAXCHARS + 5) as usize],
            // string_buffer: [" ".as_bytes()[0]; (LEDDISPLAY_MAXCHARS + 1) as usize],
             display_string: [b' ' as u8; (LEDDISPLAY_MAXCHARS + 5) as usize ],
            //display_string: String::new(),
            cursor_pos: 0,
            dot_register: [0b0000_0000; (LEDDISPLAY_MAXCHARS * 5) as usize ],
        };

        //display.display_string = String::from("    "); 

        if display.display_length > LEDDISPLAY_MAXCHARS
        {
            display.display_length = LEDDISPLAY_MAXCHARS;
        }
       
        display
    }

    // Initialize the display.
    pub fn begin(&mut self, systick: &mut bsp::SysTick,)
    {
        // info!("begin led display");
        // for number in 0..self.dot_register.len()
        // {
        //     info!("dot register {} = {} ", number, self.dot_register[number] );
        // }
        // reset the display:
        self.reset_pin.set_low().ok();
        systick.delay(10);
        self.reset_pin.set_high().ok();

        self.register_select.set_low().ok();

        self.clock_pin.set_low().ok();

        // load dot register with lows
        self.load_dot_register(systick);
        
        // set control register 0 for max brightness, and no sleep:
        self.load_all_control_registers(0b01111111,systick);
    }

    pub fn load_dot_register(&mut self, systick: &mut bsp::SysTick,)
    { 
        let max_data = self.display_length * 5;

        self.register_select.set_low().ok();

        self.enable_pin.set_low().ok();

        
        for number in 0..max_data
        {
            //info!("shifting out");
            //shiftOut(dataPin, clockPin, MSBFIRST, dotRegister[i]);
            self.shift_out(self.dot_register[number as usize], systick);
        }
            
        self.enable_pin.set_high().ok();
    }

    // /To shift out bits
    pub fn shift_out(&mut self, value: u8, systick: &mut bsp::SysTick,)
    {
        //let value = self.dotRegister[charNum];
        for i in 0..8
        {
            if value & (1 << (7-i)) > 0 {
                self.data_pin.set_high().ok();
            }
            else {
                self.data_pin.set_low().ok();
            }

            self.clock_pin.set_high().ok();
            //systick.delay(10);
            self.clock_pin.set_low().ok();
        }
    }

    // This method sends 8 bits to the control registers in all chips:
    pub fn load_all_control_registers(&mut self, data_byte: u8, systick: &mut bsp::SysTick)
    {
        // Each display can have more than one control chip, and displays
        // can be daisy-chained into long strings. For some operations, such
        // as setting the brightness, we need to ensure that a single
        // control word reaches all displays simultaneously. We do this by
        // putting each chip into simultaneous mode - effectively coupling
        // all their data-in pins together. (See section "Serial/Simultaneous
        // Data Output D0" in datasheet.)


        // One chip drives four characters, so we compute the number of
        // chips by diving by four:
        let chip_count = self.display_length / 4;

        // For each chip in the chain, write the control word that will put
        // it into simultaneous mode (seriel mode is the power-up default).
        for _i in 0..chip_count
        {
            self.load_control_registers(0b1000_0001, systick);
        }

        // Load the specified value into the control register.
        self.load_control_registers(data_byte,systick);

        // Put all the chips back into serial mode. Because they're still
        // all in simultaneous mode, we only have to write this word once.
        self.load_control_registers(0b1000_0000,systick);
    }

    // This method sends 8 bits to one of the control registers:
    fn load_control_registers(&mut self, data_byte: u8, systick: &mut bsp::SysTick)
    {
        // select the control registers:
        self.register_select.set_high().ok();
        // enable writing to the display:
        self.enable_pin.set_low().ok();
        // shift the data out:
        self.shift_out(data_byte, systick);
        // disable writing:
        self.enable_pin.set_high().ok();
    }

    pub fn clear(&mut self, systick: &mut bsp::SysTick)
    {
        self.string_buffer = [b' ' as u8; (LEDDISPLAY_MAXCHARS + 5) as usize];
        for display_pos in 0..self.display_length
        {
            self.write_character(' ', display_pos as i8);
        }

        self.load_dot_register(systick);
    }


    pub fn write_character(&mut self, what_char: char, what_position: i8)
    {
        //info!("writing led display character {} to position {}", what_char, what_position);
        // calculate the starting position in the array.
        // every character has 5 columns made of 8 bits:
        let this_position = what_position * 5;
        //info!("writing char {} to position {}", what_char, this_position);
        // copy the appropriate bits into the dot register array:
        for i in 0..5
        {
            let actual_position = i as i8 + this_position;
            self.dot_register[actual_position as usize] = FONT5X7[(((what_char as u16 - 32) * 5) + i) as usize];
            //info!("Font selector is {}", (((what_char as u16 - 32) * 5) + i) as usize);
            //info!("{} as number is supposed to be {}", what_char, what_char as u8);
            //info!("actual position is {}, value is {}", actual_position, self.dot_register[actual_position as usize]);
        }
    }

    // set the cursor to the home position (0)
    pub fn home(&mut self)
    {
        //info!("setting led display cursor to 0");
        self.cursor_pos = 0;
    }

    // set the cursor anywhere
    fn set_cursor(&mut self, which_position: i8)
    {
        self.cursor_pos = which_position;
    }

    pub fn write(&mut self, byte: char, systick: &mut bsp::SysTick)
    {
        // make sure cursorPos is on the display:
        if self.cursor_pos >= 0 && self.cursor_pos < self.display_length as i8
        {
            // put the character into the dot register:
            self.write_character(byte, self.cursor_pos);
            // put the character into the displayBuffer
            // but do not write the string constants pass
            // to us from the user by setString()

            if self.display_string == self.string_buffer && self.cursor_pos < LEDDISPLAY_MAXCHARS as i8
            {
                self.string_buffer[self.cursor_pos as usize] = byte as u8;
            }
            self.cursor_pos += 1 ;

            self.load_dot_register(systick);
        }
    }

    pub fn set_brightness(&mut self, bright: u8, systick: &mut bsp::SysTick) 
    {
        info!("set led display brightness to {}", bright);
        // Limit the brightness
        let mut brightness = bright;
        if brightness > 15 
        {
            brightness = 15;
        }
        // set the brightness:
        //info!("seting brightness to {} with byte value {}",brightness, 0b01110000 + brightness);
        self.load_all_control_registers(0b01110000 + brightness, systick);
    }

    pub fn set_string(&mut self, string_to_show: &str)
    {
        let mut i: usize = 0;
        for letter in string_to_show.bytes()
        {
            self.display_string[i] = letter;
            i += 1;
        }
        for blank in 0..self.display_string.len()-i
        {
            self.display_string[i] = 0;
            i += 1;
        }       
    }

    pub fn get_string_length(&mut self) -> usize
    {
        let mut len = 0;
        for letter in self.display_string
        {
            if letter > 32
            {
                len += 1;
            }
        }

        len
    }

    pub fn get_string_in_u8(&mut self) -> [u8; (LEDDISPLAY_MAXCHARS + 5) as usize ]
    {
        self.display_string
    }

    // Scroll the displayString across the display.  left = -1, right = +1
    pub fn scroll(&mut self, direction: bool, systick: &mut bsp::SysTick)
    {
        if direction{
            self.cursor_pos += 1;
        }
        else {
            self.cursor_pos -= 1;
        }
        //  length of the string to display:
        let string_end = self.get_string_length();
        //info!("string length is {}", string_end);

        // Loop over the string and take displayLength characters to write to the display:
        for display_pos in 0..self.display_length as usize
        {
            // which character in the strings you want:
            let which_character: i16 = display_pos as i16 - self.cursor_pos as i16;
            // which character you want to show from the string:
            let char_to_show =
            if which_character >= 0 && which_character < string_end as i16
            {
                // self.display_string.chars().nth(which_character as usize).unwrap()
                self.display_string[which_character as usize] as char
            }
            else{
                ' '
            };
            //info!("char to write is {} at pos {}", char_to_show, display_pos);
            self.write_character(char_to_show, display_pos as i8);
        }

        self.load_dot_register(systick);
    }

    pub fn get_cursor(&mut self) -> i8
    {
        self.cursor_pos
    }

    pub fn show_display_length_worth(&mut self, string_to_show: &str, systick: &mut bsp::SysTick)
    {
        self.set_string(string_to_show);
        //  length of the string to display:
        let string_end = self.get_string_length();
        
        self.cursor_pos = self.display_length as i8 - string_end as i8;

        for display_pos in 0..self.display_length as usize
        {
            // which character in the strings you want:
            let which_character: i16 = display_pos as i16 - self.cursor_pos as i16;
            // which character you want to show from the string:
            let char_to_show =
            if which_character >= 0  && which_character < string_end as i16
            {
                // self.display_string.chars().nth(which_character as usize).unwrap()
                self.display_string[which_character as usize] as char
            }
            else{
                ' '
            };
            //info!("char to write is {} at pos {}", char_to_show, display_pos);
            self.write_character(char_to_show, display_pos as i8);
        }

        self.load_dot_register(systick);
    }
}


static FONT5X7: [u8; 480] =
[
    0x00, 0x00, 0x00, 0x00, 0x00,// (space)
	0x00, 0x00, 0x5F, 0x00, 0x00,// !
	0x00, 0x07, 0x00, 0x07, 0x00,// "
	0x14, 0x7F, 0x14, 0x7F, 0x14,// #
	0x24, 0x2A, 0x7F, 0x2A, 0x12,// $
	0x23, 0x13, 0x08, 0x64, 0x62,// %
	0x36, 0x49, 0x55, 0x22, 0x50,// &
	0x00, 0x05, 0x03, 0x00, 0x00,// '
	0x00, 0x1C, 0x22, 0x41, 0x00,// (
	0x00, 0x41, 0x22, 0x1C, 0x00,// )
	0x08, 0x2A, 0x1C, 0x2A, 0x08,// *
	0x08, 0x08, 0x3E, 0x08, 0x08,// +
	0x00, 0x50, 0x30, 0x00, 0x00,// ,
	0x08, 0x08, 0x08, 0x08, 0x08,// -
	0x00, 0x60, 0x60, 0x00, 0x00,// .
	0x20, 0x10, 0x08, 0x04, 0x02,// /
	0x3E, 0x51, 0x49, 0x45, 0x3E,// 0
	0x00, 0x42, 0x7F, 0x40, 0x00,// 1
	0x42, 0x61, 0x51, 0x49, 0x46,// 2
	0x21, 0x41, 0x45, 0x4B, 0x31,// 3
	0x18, 0x14, 0x12, 0x7F, 0x10,// 4
	0x27, 0x45, 0x45, 0x45, 0x39,// 5
	0x3C, 0x4A, 0x49, 0x49, 0x30,// 6
	0x01, 0x71, 0x09, 0x05, 0x03,// 7
	0x36, 0x49, 0x49, 0x49, 0x36,// 8
	0x06, 0x49, 0x49, 0x29, 0x1E,// 9
	0x00, 0x36, 0x36, 0x00, 0x00,// :
	0x00, 0x56, 0x36, 0x00, 0x00,// ;
	0x00, 0x08, 0x14, 0x22, 0x41,// <
	0x14, 0x14, 0x14, 0x14, 0x14,// =
	0x41, 0x22, 0x14, 0x08, 0x00,// >
	0x02, 0x01, 0x51, 0x09, 0x06,// ?
	0x32, 0x49, 0x79, 0x41, 0x3E,// @
	0x7E, 0x11, 0x11, 0x11, 0x7E,// A
	0x7F, 0x49, 0x49, 0x49, 0x36,// B
	0x3E, 0x41, 0x41, 0x41, 0x22,// C
	0x7F, 0x41, 0x41, 0x22, 0x1C,// D
	0x7F, 0x49, 0x49, 0x49, 0x41,// E
	0x7F, 0x09, 0x09, 0x01, 0x01,// F
	0x3E, 0x41, 0x41, 0x51, 0x32,// G
	0x7F, 0x08, 0x08, 0x08, 0x7F,// H
	0x00, 0x41, 0x7F, 0x41, 0x00,// I
	0x20, 0x40, 0x41, 0x3F, 0x01,// J
	0x7F, 0x08, 0x14, 0x22, 0x41,// K
	0x7F, 0x40, 0x40, 0x40, 0x40,// L
	0x7F, 0x02, 0x04, 0x02, 0x7F,// M
	0x7F, 0x04, 0x08, 0x10, 0x7F,// N
	0x3E, 0x41, 0x41, 0x41, 0x3E,// O
	0x7F, 0x09, 0x09, 0x09, 0x06,// P
	0x3E, 0x41, 0x51, 0x21, 0x5E,// Q
	0x7F, 0x09, 0x19, 0x29, 0x46,// R
	0x46, 0x49, 0x49, 0x49, 0x31,// S
	0x01, 0x01, 0x7F, 0x01, 0x01,// T
	0x3F, 0x40, 0x40, 0x40, 0x3F,// U
	0x1F, 0x20, 0x40, 0x20, 0x1F,// V
	0x7F, 0x20, 0x18, 0x20, 0x7F,// W
	0x63, 0x14, 0x08, 0x14, 0x63,// X
	0x03, 0x04, 0x78, 0x04, 0x03,// Y
	0x61, 0x51, 0x49, 0x45, 0x43,// Z
	0x00, 0x00, 0x7F, 0x41, 0x41,// [
	0x02, 0x04, 0x08, 0x10, 0x20,// "\"
	0x41, 0x41, 0x7F, 0x00, 0x00,// ]
	0x04, 0x02, 0x01, 0x02, 0x04,// ^
	0x40, 0x40, 0x40, 0x40, 0x40,// _
	0x00, 0x01, 0x02, 0x04, 0x00,// `
	0x20, 0x54, 0x54, 0x54, 0x78,// a
	0x7F, 0x48, 0x44, 0x44, 0x38,// b
	0x38, 0x44, 0x44, 0x44, 0x20,// c
	0x38, 0x44, 0x44, 0x48, 0x7F,// d
	0x38, 0x54, 0x54, 0x54, 0x18,// e
	0x08, 0x7E, 0x09, 0x01, 0x02,// f
	0x08, 0x14, 0x54, 0x54, 0x3C,// g
	0x7F, 0x08, 0x04, 0x04, 0x78,// h
	0x00, 0x44, 0x7D, 0x40, 0x00,// i
	0x20, 0x40, 0x44, 0x3D, 0x00,// j
	0x00, 0x7F, 0x10, 0x28, 0x44,// k
	0x00, 0x41, 0x7F, 0x40, 0x00,// l
	0x7C, 0x04, 0x18, 0x04, 0x78,// m
	0x7C, 0x08, 0x04, 0x04, 0x78,// n
	0x38, 0x44, 0x44, 0x44, 0x38,// o
	0x7C, 0x14, 0x14, 0x14, 0x08,// p
	0x08, 0x14, 0x14, 0x18, 0x7C,// q
	0x7C, 0x08, 0x04, 0x04, 0x08,// r
	0x48, 0x54, 0x54, 0x54, 0x20,// s
	0x04, 0x3F, 0x44, 0x40, 0x20,// t
	0x3C, 0x40, 0x40, 0x20, 0x7C,// u
	0x1C, 0x20, 0x40, 0x20, 0x1C,// v
	0x3C, 0x40, 0x30, 0x40, 0x3C,// w
	0x44, 0x28, 0x10, 0x28, 0x44,// x
	0x0C, 0x50, 0x50, 0x50, 0x3C,// y
	0x44, 0x64, 0x54, 0x4C, 0x44,// z
	0x00, 0x08, 0x36, 0x41, 0x00,// {
	0x00, 0x00, 0x7F, 0x00, 0x00,// |
	0x00, 0x41, 0x36, 0x08, 0x00,// }
	0x08, 0x08, 0x2A, 0x1C, 0x08,// ->
	0x08, 0x1C, 0x2A, 0x08, 0x08 // <-
];