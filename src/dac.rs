use crate::gpio::gpioa::{PA4, PA5};
use crate::gpio::Analog;
use crate::stm32::{DAC as DAC1, RCC};
use core::marker::PhantomData;

pub unsafe trait Ch1OutPin<DAC> {}
pub unsafe trait Ch2OutPin<DAC> {}

unsafe impl Ch1OutPin<DAC1> for PA4<Analog> {}
unsafe impl Ch2OutPin<DAC1> for PA5<Analog> {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Channel {
    One,
    Two,
}

/// DAC can be use as in for dual channel access or
/// split() into Ch1 and Ch2
pub struct Dac<DAC, PINS> {
    dac: DAC,
    _pins: PINS,
}

/// Independant channel
pub struct DacChannel<DAC> {
    _dac: PhantomData<DAC>,
    ch: Channel,
}

impl<CH1PIN, CH2PIN> Dac<DAC1, (CH1PIN, CH2PIN)> {
    pub fn dac1(dac: DAC1, pins: (CH1PIN, CH2PIN)) -> Self
    where
        CH1PIN: Ch1OutPin<DAC1>,
        CH2PIN: Ch2OutPin<DAC1>,
    {
        // NOTE: only unsafe during setup
        unsafe {
            let rcc = &(*RCC::ptr());
            rcc.apb1enr.modify(|_, w| w.dac1en().enabled());
            rcc.apb1rstr.modify(|_, w| w.dac1rst().reset());
            rcc.apb1rstr.modify(|_, w| w.dac1rst().clear_bit());
        }

        dac.cr.modify(|_, w|
            // Ch2 trigger disabled
            w.ten2()
                .clear_bit()
                // Ch2 output buffered
                .boff2()
                .clear_bit()
                // Ch1 trigger disabled
                .ten1()
                .clear_bit()
                // Ch1 output buffered
                .boff1()
                .clear_bit());

        // Enable channel 1 and 2
        dac.cr.modify(|_, w| w.en2().set_bit().en1().set_bit());

        Dac { dac, _pins: pins }
    }

    pub fn set_value(&mut self, ch: Channel, val: u16) {
        // NOTE: assumes 12-bit, right aligned
        match ch {
            Channel::One => self
                .dac
                .dhr12r1
                .modify(|_, w| unsafe { w.dacc1dhr().bits(val & 0xFFF) }),
            Channel::Two => self
                .dac
                .dhr12r2
                .modify(|_, w| unsafe { w.dacc2dhr().bits(val & 0xFFF) }),
        }
    }

    pub fn split(self) -> (DacChannel<DAC1>, DacChannel<DAC1>) {
        (
            DacChannel {
                _dac: PhantomData,
                ch: Channel::One,
            },
            DacChannel {
                _dac: PhantomData,
                ch: Channel::Two,
            },
        )
    }
}

impl DacChannel<DAC1> {
    pub fn set_value(&mut self, val: u16) {
        let dac = unsafe { &(*DAC1::ptr()) };

        // NOTE: assumes 12-bit, right aligned
        match self.ch {
            Channel::One => dac
                .dhr12r1
                .modify(|_, w| unsafe { w.dacc1dhr().bits(val & 0xFFF) }),
            Channel::Two => dac
                .dhr12r2
                .modify(|_, w| unsafe { w.dacc2dhr().bits(val & 0xFFF) }),
        }
    }
}
