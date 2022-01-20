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
    use usbd_serial::SerialPort;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        usb_dev: UsbDevice<'static, UsbBus>,
        serial: SerialPort<'static, UsbBus>,
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
        let serial = SerialPort::new(usb_bus.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(usb_bus.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(2)
            .build();

        info!("Send me a string!");
        (Shared {}, Local { usb_dev, serial }, init::Monotonics())
    }

    #[task(binds=USBCTRL_IRQ, local = [serial, usb_dev])]
    fn on_usb(ctx: on_usb::Context) {
        let serial = ctx.local.serial;
        if !ctx.local.usb_dev.poll(&mut [serial]) {
            return;
        }
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                info!("Received: {}", core::str::from_utf8(&buf[..]).unwrap());
                buf.iter_mut().take(count).for_each(|b| {
                    b.make_ascii_uppercase();
                });
                // Echo back to the host
                let mut wr_ptr = &buf[..count];
                while !wr_ptr.is_empty() {
                    let _ = serial.write(wr_ptr).map(|len| {
                        wr_ptr = &wr_ptr[len..];
                    });
                }
            }
            _ => {}
        }
    }
}
