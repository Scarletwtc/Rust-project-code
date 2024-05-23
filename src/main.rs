#![no_std]
#![no_main]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{PIN_5, PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_19, PIN_20, PIN_28};
use embassy_rp::usb::{Driver, InterruptHandler, Out};
use embassy_rp::{bind_interrupts, peripherals::USB};
use log::info;
use embassy_rp::spi::{self, Spi};
use embassy_time::{Duration, Timer};
use mfrc522::{Mfrc522};
use core::time;
use embassy_rp::gpio::{Level, Output, Pull, Input};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};


// use cortex_m::prelude::{
//     _embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs,
// };

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn fire_task(mut fire_sensor: Input<'static, PIN_5>, mut buzzer: Pwm<'static, embassy_rp::peripherals::PWM_CH5>) {
    let mut config_pwm: PwmConfig = Default::default();
    config_pwm.top = 0xFFFF;
    config_pwm.compare_b = config_pwm.top;

    loop {
        fire_sensor.wait_for_rising_edge().await; 
        config_pwm.compare_b = config_pwm.top;
        buzzer.set_config(&config_pwm);
        fire_sensor.wait_for_low().await;
        config_pwm.compare_b = config_pwm.top / 2; 
        buzzer.set_config(&config_pwm);             
    }
}
#[embassy_executor::task]
async fn park1_task(mut ir_sensor1: Input<'static, PIN_6>, mut led_red1: Output<'static, PIN_12>, mut led_green1: Output<'static, PIN_13>) {
    loop {
        if ir_sensor1.is_high() {
            led_red1.set_high();
            led_green1.set_low();
        } else {
            led_red1.set_low();
            led_green1.set_high();
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
#[embassy_executor::task]
async fn park2_task(mut ir_sensor2: Input<'static, PIN_8>, mut led_red2: Output<'static, PIN_14>, mut led_green2: Output<'static, PIN_15>) {
    loop {
        if ir_sensor2.is_high() {
            led_red2.set_high();
            led_green2.set_low();
        } else {
            led_red2.set_low();
            led_green2.set_high();
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
#[embassy_executor::task]
async fn park3_task(mut ir_sensor3: Input<'static, PIN_7>, mut led_red3: Output<'static, PIN_16>, mut led_green3: Output<'static, PIN_17>) {
    loop {
        if ir_sensor3.is_high() {
            led_red3.set_high();
            led_green3.set_low();
        } else {
            led_red3.set_low();
            led_green3.set_high();
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}

#[embassy_executor::task]
async fn door1_task(mut ir_sensor_door1: Input<'static, PIN_10>, mut servo1: Output<'static, PIN_20>) {
    loop {
        ir_sensor_door1.wait_for_rising_edge().await;
        //ibla3333333333333i
        //abd is always right
        info!("Infrared sensor is high");
        info!("Infrared sensor is high");
        servo1.set_high();
        Timer::after(Duration::from_millis(1)).await;
        servo1.set_low();
        Timer::after(Duration::from_secs(1 as u64)).await;
        servo1.set_high();
        Timer::after(Duration::from_millis(1)).await;
        servo1.set_low();
        Timer::after(Duration::from_secs(1 as u64)).await;
        servo1.set_high();
        Timer::after(Duration::from_millis(1)).await;
        servo1.set_low();
        Timer::after(Duration::from_secs(1 as u64)).await;

        Timer::after_secs(5).await;
        // Set PWM signal for 180 degrees (assume 2 ms pulse width)
        servo1.set_high();
        Timer::after(Duration::from_millis(2)).await;
        servo1.set_low();
        Timer::after(Duration::from_secs(1 as u64)).await;

        servo1.set_high();
        Timer::after(Duration::from_millis(2)).await;
        servo1.set_low();
        Timer::after(Duration::from_secs(1 as u64)).await;
        servo1.set_high();
        Timer::after(Duration::from_millis(2)).await;
        servo1.set_low();
        Timer::after(Duration::from_secs(1 as u64)).await;

    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    //rfid 
    let miso = peripherals.PIN_4;
    let mosi = peripherals.PIN_3;
    let sck = peripherals.PIN_2;
    let rst = peripherals.PIN_0;
    let sda = peripherals.PIN_1;
    
    let mut led_rfid= Output::new(peripherals.PIN_27, Level::Low);

    let mut cs = Output::new(sda, Level::Low);
    let mut reset = Output::new(rst, Level::High);
    reset.set_low();
    Timer::after_millis(10).await;
    reset.set_high();

    let mut spi_config = spi::Config::default();
    spi_config.frequency = 1_000_000;
    spi_config.polarity = spi::Polarity::IdleLow;
    spi_config.phase = spi::Phase::CaptureOnFirstTransition;

    let mut spi = Spi::new(peripherals.SPI0, sck, mosi, miso, peripherals.DMA_CH0, peripherals.DMA_CH1, spi_config);

    let mut mfrc = Mfrc522::new(spi).with_nss(cs).init().unwrap();

    let known_uids: [[u8; 4]; 2] = [
    [0xAA, 0xBB, 0xCC, 0xDD], // Existing example UID
    [99, 17, 24, 166],        // New known UID
    ];

    //infrared sensors+ leds
    let ir_sensor1 = Input::new(peripherals.PIN_6, Pull::Up);
    let mut led_red1 = Output::new(peripherals.PIN_12, Level::Low);
    let mut led_green1 = Output::new(peripherals.PIN_13, Level::Low);
    let ir_sensor3 = Input::new(peripherals.PIN_7, Pull::Up);
    let mut led_red2 = Output::new(peripherals.PIN_14, Level::Low);
    let mut led_green2 = Output::new(peripherals.PIN_15, Level::Low);
    let ir_sensor2 = Input::new(peripherals.PIN_8, Pull::Up);
    let mut led_red3 = Output::new(peripherals.PIN_16, Level::Low);
    let mut led_green3 = Output::new(peripherals.PIN_17, Level::Low);
    let ir_sensor_door2 = Input::new(peripherals.PIN_9, Pull::Up);
    let ir_sensor_door1 = Input::new(peripherals.PIN_10, Pull::Up);


    //fire sensor 
    let fire_sensor = Input::new(peripherals.PIN_5, Pull::Up);

    //buzzer 
    let mut config_pwm: PwmConfig = Default::default();
    config_pwm.top = 0xFFFF;
    config_pwm.compare_b = config_pwm.top;


    let mut config_pwm2: PwmConfig = Default::default();
    config_pwm2.top = 0xFFFF;
    config_pwm2.compare_a = config_pwm2.top;

    //buzzers
    let mut buzzer = Pwm::new_output_b(peripherals.PWM_CH5, peripherals.PIN_11, config_pwm.clone());
    let mut buzzer2= Pwm::new_output_a(peripherals.PWM_CH6, peripherals.PIN_28, config_pwm2.clone());
    //servos
    let mut servo1 = Output::new(peripherals.PIN_20, Level::Low);
    let mut servo2 = Output::new(peripherals.PIN_19, Level::Low);

    //spawning tasks
    spawner.spawn(fire_task(fire_sensor, buzzer)).unwrap();
    spawner.spawn(park1_task(ir_sensor1, led_red1, led_green1)).unwrap();
    spawner.spawn(park2_task(ir_sensor2, led_red2, led_green2)).unwrap();
    spawner.spawn(park3_task(ir_sensor3, led_red3, led_green3)).unwrap();
    let _= spawner.spawn(door1_task(ir_sensor_door1, servo1));
    //let _= spawner.spawn(door2_task(ir_sensor_door2, servo2));


    loop {
        match mfrc.new_card_present() {
            Ok(atqa) => { // This means a card is present
                if let Ok(uid) = mfrc.select(&atqa) { // Attempt to select the card
                    let uid_bytes = uid.as_bytes(); // Assuming method bytes() returns UID bytes   
                    // Logging the UID for debugging or display
                    info!("Card UID: {:?}", uid_bytes);
    
                    let is_known = known_uids.iter().any(|&k| k == uid_bytes);
                    if is_known {
                        info!("Known card detected!"); 
                        led_rfid.set_high();
                    } else {
                        info!("Unknown card detected!");
                        config_pwm2.compare_a = config_pwm2.top/2;
                        buzzer2.set_config(&config_pwm2);
                    }
                     
                }
            },
            Err(e) => {
                //info!("Error checking for new card: {:?}", e);
            }
        }
        Timer::after(Duration::from_millis(200)).await;
        config_pwm2.compare_a = config_pwm2.top; 
        buzzer2.set_config(&config_pwm2);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
