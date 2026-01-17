#![no_std]
#![no_main]

use crate::app::Application;
use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_stm32::SharedData;

use {defmt_rtt as _, panic_probe as _};

mod app;
mod co_fut;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = {
        use embassy_stm32::rcc::*;

        let mut config = embassy_stm32::Config::default();
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;

        // Required to for ST-Link to work on the Nucleo board, see warning at the end of Section
        // 7.4.8 "Internal SMPS/LDO Configuration" in UM2408 Rev 6.
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config
    };

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);

    let app = Application::new(p);
    app.run().await;

    unreachable!()
}
