#![no_std]
#![no_main]

#[allow(unused_imports)]
extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::{hprint, hprintln};
use stm32f3::stm32f303;

//defining megnetometer address as constant
const MEG_ADD: u16 = 0b001_1110;

// Addresses of the magnetometer's registers
//const OUT_X_H_M: u8 = 0x03;
const IRA_REG_M: u8 = 0x0A;

#[entry]
fn main() -> ! {
    
    let peripherals = stm32f3::stm32f303::Peripherals::take().unwrap();
    let i2c = peripherals.I2C1;
    let rcc = peripherals.RCC;
    let portb = peripherals.GPIOB;

    //**************pin setting*********************//
    rcc.ahbenr.write(|w| unsafe{
        w  
            .iopben().set_bit()
    });

    portb.moder.write(|w| unsafe{
        w  
            .moder6().bits(0b01) //alternate function
            .moder7().bits(0b01)
    });

    portb.pupdr.write(|w| unsafe{
        w  
            .pupdr6().bits(0b10) //pullup + pulldown
            .pupdr7().bits(0b10)
    });
    portb.ospeedr.write(|w| unsafe{
        w  
            .ospeedr6().bits(0b10) //output speed medium
            .ospeedr7().bits(0b10)
    });

    //**************setting clock for i2c ***********/
    rcc.apb1enr.write(
        |w| w.i2c1en().set_bit(), //enabling 12c1 clock
    );

    rcc.cfgr3.write(
        |w| w.i2c1sw().set_bit(), //configuring i2c clock souce as sysclk
    );

    //***********configuring and setting clock for i2c ******************/
    i2c.timingr.write(|w| unsafe {
        w.presc()
            .bits(0b0001) //the clock period t(PRESC) used for data setup and hold counters
            .scll()
            .bits(0xc7) //SCL low period
            .sclh()
            .bits(0xc3) //SCL high period
            .sdadel()
            .bits(0x2) //delay t(SDADEL) between SCL falling edge and SDA edge
            .scldel()
            .bits(0x4) //delay t(SCLDEL) between SDA edge and SCL rising edge
    });

    i2c.cr1.write(|w| {
        w.nostretch()
            .clear_bit() //kept cleared in master mode
            .pe()
            .set_bit()
    });

    //******************************************************/
    //now after initializing i2c we will be dealing with following registers:
    //1--> CR2 Control register 2.
    //broadcast the adress of the megnetometer we want to work with:
    i2c.cr2.write(|w| unsafe {
        w.start()
            .set_bit() //start broadcast
            .sadd()
            .bits(MEG_ADD) //megnitometer address setting
            .rd_wrn()
            .clear_bit() //Master requests a write transfer.
            .nbytes()
            .bits(1) //The number of bytes to be transmitted/received
            .autoend()
            .clear_bit() //manual end selection
    });

    cortex_m::asm::delay(8_000_000);

    //2--> ISR. Interrupt and status register.
    //wait untill we can send more data
    while i2c.isr.read().txis().bit_is_clear() {}

    //3--> TXDR. Transmit data register.
    //send the address of the register we want to read
    i2c.txdr.write(|w| unsafe { w.txdata().bits(IRA_REG_M) });
    //wait untill previouse byte has been transmitted
    while i2c.isr.read().tc().bit_is_clear() {}

    //4--> RXDR. Receive data register
    //let recive the data from the register we ask for
    let bytes = {
        i2c.cr2.write(|w| unsafe {
            w.start()
                .set_bit() //start broadcast
                .rd_wrn()
                .set_bit() //Master requests a write transfer.
                .nbytes()
                .bits(1) //The number of bytes to be transmitted/received
                .autoend()
                .set_bit() //manual end selection
        });

        // Wait until we have received the contents of the register
        while i2c.isr.read().rxne().bit_is_clear() {} //if register not empty

        i2c.rxdr.read().rxdata().bits()
    };

    hprintln!("adress: {}   bytes:{}", IRA_REG_M, bytes);

    loop {}
}
