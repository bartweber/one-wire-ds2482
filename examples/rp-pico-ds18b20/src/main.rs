#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use ds18b20::{Ds18b20, Resolution};
use one_wire_hal::OneWire;
use panic_halt as _;
use rp_pico::{entry, hal, pac};
use rp_pico::hal::Clock;
use rp_pico::hal::fugit::RateExtU32;
use rp_pico::hal::gpio::FunctionI2C;
use rtt_target::{rprintln, rtt_init_print};

use one_wire_ds2482::OneWireDS2482;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    // set up delay provider
    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().raw());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sda_pin: hal::gpio::Pin<_, FunctionI2C, _> = pins.gpio2.reconfigure();
    let scl_pin: hal::gpio::Pin<_, FunctionI2C, _> = pins.gpio3.reconfigure();

    let i2c = hal::I2C::i2c1(
        pac.I2C1,
        sda_pin,
        scl_pin,
        500.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    rtt_init_print!();

    // setup one-wire bus
    rprintln!("setup one-wire (DS2482) bus...");
    let mut ds2482 = OneWireDS2482::new(i2c, 0x18);
    rprintln!("ds2482 created");
    ds2482.ds2482_device_reset().unwrap();
    rprintln!("ds2482 reset");
    ds2482.ds2482_write_config(0b0001).unwrap();
    rprintln!("ds2482 configured");
    let mut one_wire = ds2482;

    let addr_1 = {
        // search for devices
        rprintln!("searching for devices...");
        let mut devices = one_wire.devices(&mut delay);
        rprintln!("devices iterator created");
        let addr = devices.next().unwrap().unwrap();
        rprintln!("found device on address: {:?}", addr);

        let other_addr = devices.next();
        if other_addr.is_none() {
            rprintln!("no more devices found");
        } else {
            rprintln!("found yet another device: {:?}; ignoring it", other_addr.unwrap().unwrap());
        }

        addr
    };

    // temp device #1
    let temp_sensor_1 = Ds18b20::new(addr_1).unwrap();

    // read temperature
    loop {
        ds18b20::start_simultaneous_temp_measurement(&mut one_wire, &mut delay).unwrap();
        Resolution::Bits12.delay_for_measurement_time(&mut delay);
        let data_1 = temp_sensor_1.read_data(&mut one_wire, &mut delay).unwrap();

        let temp_1 = data_1.temperature;

        rprintln!("temp 1: {:?}", temp_1);

        delay.delay_ms(2000);
    }
}