//! The starter code slowly blinks the LED, and sets up
//! USB logging.

#![no_std]
#![no_main]

use cortex_m::peripheral::syst::SystClkSource;
use teensy4_bsp as bsp;
use teensy4_panic as _;
use core::time::Duration;

mod logging;

const LED_PERIOD: Duration = Duration::from_millis(1_000);
/// The GPT output compare register we're using for
/// tracking time. This is the first register, since
/// we're using reset mode.
const GPT_OCR: bsp::hal::gpt::OutputCompareRegister = bsp::hal::gpt::OutputCompareRegister::One;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut periphs = bsp::Peripherals::take().unwrap();
    let mut cortex = cortex_m::Peripherals::take().unwrap();

    // Reduce the number of pins to those specific
    // to the Teensy 4.0.
    let pins = bsp::pins::t40::from_pads(periphs.iomuxc);
    // Prepare the LED, and turn it on!
    // (If it never turns off, something
    // bad happened.)
    let mut led = bsp::configure_led(pins.p13);
    led.set();

    // Prepare the ARM clock to run at ARM_HZ.
    periphs.ccm.pll1.set_arm_clock(
        bsp::hal::ccm::PLL1::ARM_HZ,
        &mut periphs.ccm.handle,
        &mut periphs.dcdc,
    );

    // Prepare a GPT timer for blocking delays.
    let mut timer = {
        // Run PERCLK on the crystal oscillator (24MHz).
        let mut cfg = periphs.ccm.perclk.configure(
            &mut periphs.ccm.handle,
            bsp::hal::ccm::perclk::PODF::DIVIDE_1,
            bsp::hal::ccm::perclk::CLKSEL::OSC,
        );

        let mut gpt1 = periphs.gpt1.clock(&mut cfg);
        // Keep ticking if we enter wait mode.
        gpt1.set_wait_mode_enable(true);
        // When the first output compare register compares,
        // reset the counter back to zero.
        gpt1.set_mode(bsp::hal::gpt::Mode::Reset);

        // Compare every LED_PERIOD_US ticks.
        gpt1.set_output_compare_duration(GPT_OCR, LED_PERIOD);
        gpt1
    };

    // See the `logging` module docs for more info.
    assert!(logging::init().is_ok());

    log::info!("Starting test...");

    // cortex.SYST.set_clock_source(SystClkSource::Core);
    // cortex.SYST.set_reload(0x00ffffff);
    // cortex.SYST.enable_counter();

    cortex.DCB.enable_trace();
    cortex.DWT.enable_cycle_counter();

    let mut x = 0;

    for i in 0..=1_000_000 {
        x += i;
        log::info!("{x}");
    }

    let dur = cortex_m::peripheral::DWT::cycle_count();

    

    timer.set_enable(true);
    loop {
        led.toggle();
        //log::info!("Hello world");
        log::info!("Took {dur} ticks: {x}, {}", cortex_m::peripheral::SYST::get_current());
        while !timer.output_compare_status(GPT_OCR).is_set() {}
        timer.output_compare_status(GPT_OCR).clear();
    }
}
