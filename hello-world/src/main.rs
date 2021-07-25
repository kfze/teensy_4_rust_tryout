#![no_std]
#![no_main]

// To build: cargo objcopy --release -- -O ihex led_display.hex

use bsp::{t40::{into_pins}};
use teensy4_bsp as bsp;
use teensy4_panic as _;
use log::{info};
use heapless::String;

use crate::{ir::{configure_an1, configure_ir, get_ir1}, led_display::Hcms};
// use crate::ir::{configure_ir};

mod logging;
mod ir;
//pub mod irV2;
mod led_display;


//const LED_PERIOD_MS: u32 = 500;
const LED_DISPLAY_LENGTH: u8 = 4;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut p = bsp::Peripherals::take().unwrap();
    let mut systick = bsp::SysTick::new(cortex_m::Peripherals::take().unwrap().SYST);
    let pins = into_pins(p.iomuxc);

    let mut led = bsp::configure_led(pins.p13);

    // Initialise IR led on Pin 11 
    let mut ir1 = configure_ir(pins.p11);
    //Initialise Analog input on Pin 14
    let (adc1_builder, _) = p.adc.clock(&mut p.ccm.handle);
    let mut adc1 = adc1_builder.build(bsp::hal::adc::ClockSelect::default(), bsp::hal::adc::ClockDivision::default());
    //let mut a1 = bsp::hal::adc::AnalogInput::new(pins.p14);
    let mut a1 = configure_an1(pins.p14);

    let data_pin = bsp::hal::gpio::GPIO::new(pins.p6);
    let mut data_pin = data_pin.output();
    let register_select_pin = bsp::hal::gpio::GPIO::new(pins.p7);
    let mut register_select_pin = register_select_pin.output();
    let clock_pin = bsp::hal::gpio::GPIO::new(pins.p8);
    let mut clock_pin = clock_pin.output();
    let enable_pin = bsp::hal::gpio::GPIO::new(pins.p9);
    let mut enable_pin = enable_pin.output();
    let reset_pin = bsp::hal::gpio::GPIO::new(pins.p10);
    let mut reset_pin = reset_pin.output();

    let mut led_display = Hcms::new_display(data_pin, register_select_pin,clock_pin,enable_pin,reset_pin, LED_DISPLAY_LENGTH);

    // See the `logging` module docs for more info.
    assert!(logging::init().is_ok());

    systick.delay(1000);
    info!("starting led_display");
    led_display.begin(&mut systick);
    led_display.set_brightness(2, &mut systick);
    led_display.home();
    //let mut led_direction = true;

    // let char_used = '1';
    // info!("writing char {}", char_used as u8);
    // led_display.write('1', &mut systick);
    // led_display.write('2', &mut systick);
    // for i in 0..4{
    //     let value = i as u8 + '0' as u8;
    //     info!("value to be shown: {}", value);
    //     // led_display.write(value as char, &mut systick);
    //     led_display.write('h' , &mut systick);
    // }

    loop {
        
        let reading_a1 =  get_ir1(&mut systick, &mut ir1, &mut a1, &mut adc1);
        info!("ir1: {}", reading_a1);
        let s: String<16> = String::from(reading_a1);
        // led_display.set_string(&s);
        // led_display.scroll(true, &mut systick);
        led_display.show_display_length_worth(&s, &mut systick);
        if reading_a1 > 100 {
            led.set();
        }
        else {
            led.clear();
        }
        //systick.delay(100);
        
        // LED display Tests
        // led_display.set_string("hello");
        // //info!("cursor location is {}, direction is {}", led_display.get_cursor(), led_direction);
        // if led_display.get_cursor() > LED_DISPLAY_LENGTH as i8 || led_display.get_cursor() <= -(led_display.get_string_length() as i8) 
        // {
        //     //info!("cursor location is {}", led_display.get_cursor());
        //     led_direction = !led_direction;
        //     systick.delay(1000);
        //     //info!("suppose to reverse, cursor location is {}, direction is {}", led_display.get_cursor(), led_direction);
        // }
        // led_display.scroll(led_direction, &mut systick);
        // systick.delay(100);
    }
}



