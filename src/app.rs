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
        loop {
            embassy_time::Timer::after_secs(60).await;
        }
    }
}

#[interrupt]
fn TIM2() {
    static IS_ENABLED: AtomicBool = AtomicBool::new(false);

    let is_enabled = IS_ENABLED.load(Ordering::Relaxed);
    IS_ENABLED.store(!is_enabled, Ordering::Relaxed);
    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs)
            .as_mut()
            // INVARIANT: The LED has been initialized before the interrupt is enabled
            .unwrap()
            .set_level(is_enabled.into());

        TIMER
            .borrow_ref_mut(cs)
            .as_mut()
            // INVARIANT: The TIMER has been initialized before the interrupt is enabled
            .unwrap()
            .clear_update_interrupt();
    });
}
