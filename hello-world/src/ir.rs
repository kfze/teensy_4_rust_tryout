#![cfg_attr(docsrs, feature(doc_cfg))]

// Need to reference this so that it doesn't get stripped out
#[cfg(target_arch = "arm")]
extern crate teensy4_fcb;

use hal::iomuxc::adc::ADC1;
use embedded_hal::adc::OneShot;
use teensy4_bsp as bsp;
pub use teensy4_pins::common;
pub use teensy4_pins::t40;
pub use teensy4_pins::t41;

#[cfg(all(target_arch = "arm", feature = "rt"))]
mod rt;
#[cfg(feature = "systick")]
mod systick;
#[cfg(feature = "usb-logging")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb-logging")))]
pub mod usb;

#[cfg(feature = "systick")]
pub use systick::SysTick;

pub use hal::ral::interrupt;
// `rtic` expects these in the root.
#[doc(hidden)]
#[cfg(feature = "rtic")]
pub use hal::ral::{interrupt as Interrupt, NVIC_PRIO_BITS};

pub use hal::Peripherals;
pub use imxrt_hal as hal;


/// The IRs
///
//pub type IR1 = hal::gpio::GPIO<common::P13, hal::gpio::Output>;
pub type IR1 = hal::gpio::GPIO<common::P11, hal::gpio::Output>;
pub type An1 = hal::adc::AnalogInput<ADC1, common::P14>;

/// Configure the board's LED
///
/// Returns a GPIO that's physically tied to the LED. Use the returned handle
/// to drive the LED.
pub fn configure_ir(pad: common::P11) -> IR1 {
    let ir_pin = hal::gpio::GPIO::new(pad);
    let mut ir_out = ir_pin.output();
    ir_out.set_fast(true);
    ir_out
}

pub fn configure_an1(pad: common::P14) -> An1
{
    hal::adc::AnalogInput::new(pad)
}

pub fn get_ir1(systick: &mut bsp::SysTick, ir1: &mut hal::gpio::GPIO<t40::P11, hal::gpio::Output>, an1: &mut hal::adc::AnalogInput<ADC1, t40::P14>, adc1: &mut hal::adc::ADC<ADC1>) -> u16
{
    
    ir1.set();
    //ir1_output.set();
    systick.delay(50);
    let reading: u16 = adc1.read( an1).unwrap();
    //info!("ir1: {}", reading);
    // ir1_output.clear();
    
    ir1.clear();
    systick.delay(50);
    reading
}