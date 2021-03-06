#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use cortex_m_semihosting::hprintln;
use hal::analog::adc::{Precision, SampleTime, VTemp};
use hal::prelude::*;
use hal::stm32;
use rt::entry;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let cp = stm32::CorePeripherals::take().expect("cannot take core peripherals");
    let mut rcc = dp.RCC.constrain();
    let mut delay = cp.SYST.delay(&mut rcc);

    let gpioa = dp.GPIOA.split(&mut rcc);

    let mut adc = dp.ADC.constrain(&mut rcc);
    adc.set_sample_time(SampleTime::T_80);
    adc.set_precision(Precision::B_12);
    delay.delay(20.us()); // Wait for ADC voltage regulator to stabilize
    adc.calibrate();

    let mut adc_pin = gpioa.pa0.into_analog();
    let mut vtemp = VTemp::new();
    vtemp.enable(&mut adc);

    loop {
        let u_raw: u32 = adc.read(&mut adc_pin).expect("adc read failed");
        let temp: u32 = adc.read(&mut vtemp).expect("temperature read failed");

        let u = u_raw.saturating_sub(32) as f32 / 4_096_f32 * 3.3;
        let temp = temp / 42;
        hprintln!("u: {:.2} V | t: {:?}°C", u, temp).unwrap();
    }
}
