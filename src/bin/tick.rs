#![no_std]
#![no_main]

use rp_rtic as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true, dispatchers = [XIP_IRQ])]
mod app {
    use defmt::*;
    use rp2040_monotonic::*;
    use rp_pico::{
        hal::{clocks::init_clocks_and_plls, watchdog::Watchdog},
        XOSC_CRYSTAL_FREQ,
    };

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Monotonic = Rp2040Monotonic;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut resets = c.device.RESETS;
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let _clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let mono = Monotonic::new(c.device.TIMER);
        tick::spawn().ok();
        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task]
    fn tick(_: tick::Context) {
        info!("Tick");
        tick::spawn_after(1_000_u64.millis()).ok();
    }
}
