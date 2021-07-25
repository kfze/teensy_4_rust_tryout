#![cfg_attr(not(test), no_std)]

use core::time::Duration;
use cortex_m::asm::delay;
use embedded_hal::blocking::delay::{self, DelayMs};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::adc::OneShot;
use imxrt_hal::adc::{self, AnalogInput};
use imxrt_hal::gpio::Input;
use imxrt_hal::iomuxc::adc::{ADC, ADC1, Pin};

const IR_DELAY: Duration = Duration::from_millis(20);

pub struct IrUnitADC1<TransmitterPin: OutputPin, ReceiverPin: Pin<ADC1>, ADCx: ADC> {
    trans_pin: TransmitterPin,
    receive_pin: ReceiverPin,
    adc: ADCx,
}

impl<TransmitterPin: OutputPin, ReceiverPin: Pin<ADC1>, ADCx: ADC> IrUnitADC1<TransmitterPin, ReceiverPin, ADCx> {
    pub fn setup_ir(transmit_pin: TransmitterPin, receiver_pin: ReceiverPin, adcx: ADCx) -> Self{
        Self{
            trans_pin: transmit_pin,
            receive_pin: receiver_pin,
            
        }
    }

    pub fn read_ir<D: DelayMs<u8>>(&mut self, delay: &mut D) -> u16
    {
        self.trans_pin.set_high();
        delay.delay_ms(IR_DELAY.as_millis() as u8);
        let reading: u16 = 

    }
}