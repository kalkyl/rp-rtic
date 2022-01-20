#![no_std]
#![no_main]

use rp_rtic as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {
    use defmt::*;
    use rp_pico::{
        hal::{clocks::init_clocks_and_plls, usb::UsbBus, watchdog::Watchdog},
        XOSC_CRYSTAL_FREQ,
    };
    use usb_device::{class_prelude::*, prelude::*};
    use usbd_hid::descriptor::generator_prelude::*;
    use usbd_hid::descriptor::MouseReport;
    use usbd_hid::hid_class::HIDClass;

    #[shared]
    struct Shared {
        hid: HIDClass<'static, UsbBus>,
    }

    #[local]
    struct Local {
        usb_dev: UsbDevice<'static, UsbBus>,
    }

    #[init(local = [usb_bus: Option<UsbBusAllocator<UsbBus>> = None])]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut resets = c.device.RESETS;
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
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

        let usb_bus = c.local.usb_bus;
        usb_bus.replace(UsbBusAllocator::new(UsbBus::new(
            c.device.USBCTRL_REGS,
            c.device.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut resets,
        )));

        let hid = HIDClass::new(usb_bus.as_ref().unwrap(), MouseReport::desc(), 60);
        let usb_dev = UsbDeviceBuilder::new(usb_bus.as_ref().unwrap(), UsbVidPid(0xc410, 0x0000))
            .manufacturer("Fake company")
            .product("Mouse")
            .serial_number("TEST")
            .device_class(0)
            .build();

        info!("Mouse example!");
        (Shared { hid }, Local { usb_dev }, init::Monotonics())
    }

    #[idle(shared = [hid], local=[counter: u8 = 0])]
    fn idle(mut ctx: idle::Context) -> ! {
        let counter = ctx.local.counter;
        loop {
            let report = MouseReport {
                x: if *counter < 64 { 3 } else { -3 },
                y: 0,
                buttons: 0,
                wheel: 0,
                pan: 0,
            };
            ctx.shared.hid.lock(|hid| hid.push_input(&report).ok());
            *counter = (*counter + 1) % 128;
            cortex_m::asm::delay(500_000);
        }
    }

    #[task(binds=USBCTRL_IRQ, shared = [hid], local=[usb_dev])]
    fn on_usb(mut ctx: on_usb::Context) {
        let usb_dev = ctx.local.usb_dev;
        ctx.shared.hid.lock(|hid| if !usb_dev.poll(&mut [hid]) {});
    }
}
