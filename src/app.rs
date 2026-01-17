#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

use cortex_m::peripheral::NVIC;
use critical_section::Mutex;
use embassy_stm32::{
    Peripherals,
    gpio::{Level, Output, Speed},
    interrupt,
    peripherals::TIM2,
    time::hz,
    timer::low_level::Timer,
};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static LED: Mutex<RefCell<Option<Output>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<Timer<'static, TIM2>>>> = Mutex::new(RefCell::new(None));

pub struct Application;

impl Application {
    pub fn new(p: Peripherals) -> Self {
        let ld1 = Output::new(p.PB0, Level::Low, Speed::Low);

        let timer = Timer::new(p.TIM2);
        timer.set_frequency(hz(1));
        timer.start();
        timer.enable_update_interrupt(true);

        critical_section::with(|cs| {
            *LED.borrow_ref_mut(cs) = Some(ld1);
            *TIMER.borrow_ref_mut(cs) = Some(timer);
        });

        // SAFETY: Unmask interrupt after initializing global timer to preserve TIMER/LED
        // initialization invariant.
        unsafe { NVIC::unmask(interrupt::TIM2) };
        Application
    }

    pub async fn run(&self) {
        // SAFETY: app is free from U.B.
        unsafe { app() };
    }
}

#[unsafe(no_mangle)]
extern "C" fn set_led(state: bool) {
    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs)
            .as_mut()
            // INVARIANT: The LED has been initialized before the interrupt is enabled
            .unwrap()
            .set_level(state.into());
    });
}

#[interrupt]
fn TIM2() {
    static IS_READY: AtomicBool = AtomicBool::new(false);

    let is_ready = IS_READY.load(Ordering::Relaxed);
    IS_READY.store(!is_ready, Ordering::Relaxed);

    unsafe { READY = is_ready };

    critical_section::with(|cs| {
        TIMER
            .borrow_ref_mut(cs)
            .as_mut()
            // INVARIANT: The TIMER has been initialized before the interrupt is enabled
            .unwrap()
            .clear_update_interrupt();
    });
}
