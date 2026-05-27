#![no_std]
#![no_main]
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::usart::{Config as UartConfig, Uart};
use embassy_time::{Duration, Timer};
use embedded_hal::Pwm;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::i2c::I2c;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};



fn duty_index(angle: u32, max_duty: u32) -> u32 {
let pulse_us = 700 + (angle * 1500 / 180);
    (pulse_us * max_duty) / 20_000
}
fn duty_middle(angle: u32, max_duty: u32) -> u32 {
let pulse_us = 700 + (angle * 1500 / 180);
    (pulse_us * max_duty) / 20_000
}
fn duty_ring(angle: u32, max_duty: u32) -> u32 {
let pulse_us = 700 + (angle * 1500 / 180);
    (pulse_us * max_duty) / 20_000
}
fn duty_pinky(angle: u32, max_duty: u32) -> u32 {
let pulse_us = 700 + (angle * 1500 / 180);
    (pulse_us * max_duty) / 20_000
}

async fn show_rock(pwm: &mut SimplePwm<'_, embassy_stm32::peripherals::TIM3>, max: u32) {
pwm.set_duty(Channel::Ch1, duty_index(0, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch3, duty_middle(60, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch2, duty_ring(120, max));
Timer::after(Duration::from_millis(800)).await;
pwm.set_duty(Channel::Ch4, duty_pinky(180, max));
Timer::after(Duration::from_millis(1500)).await;
}
async fn show_paper(pwm: &mut SimplePwm<'_, embassy_stm32::peripherals::TIM3>, max: u32) {

pwm.set_duty(Channel::Ch2, duty_ring(60, max));
Timer::after(Duration::from_millis(600)).await;
pwm.set_duty(Channel::Ch2, duty_ring(50, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch1, duty_index(180, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch3, duty_middle(0, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch4, duty_pinky(0, max));
Timer::after(Duration::from_millis(1500)).await;
}
async fn show_scissors(pwm: &mut SimplePwm<'_, embassy_stm32::peripherals::TIM3>, max: u32) {
pwm.set_duty(Channel::Ch1, duty_index(180, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch3, duty_middle(0, max));
Timer::after(Duration::from_millis(700)).await;
pwm.set_duty(Channel::Ch2, duty_ring(120, max));
Timer::after(Duration::from_millis(800)).await;
pwm.set_duty(Channel::Ch4, duty_pinky(180, max));
Timer::after(Duration::from_millis(1500)).await;
}
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
let p = embassy_stm32::init(Default::default());

let ch1 = PwmPin::new(p.PC6, OutputType::PushPull); 
let ch2 = PwmPin::new(p.PC7, OutputType::PushPull); 
let ch3 = PwmPin::new(p.PC8, OutputType::PushPull); 
let ch4 = PwmPin::new(p.PC9, OutputType::PushPull); 
let mut pwm = SimplePwm::new(
p.TIM3,
Some(ch1),
Some(ch2),
Some(ch3),
Some(ch4),
hz(50),
CountingMode::EdgeAlignedUp,
    );
let max = pwm.get_max_duty();
pwm.enable(Channel::Ch1);
pwm.enable(Channel::Ch2);
pwm.enable(Channel::Ch3);
pwm.enable(Channel::Ch4);

pwm.set_duty(Channel::Ch3, duty_middle(0, max));
Timer::after(Duration::from_millis(1000)).await;
pwm.set_duty(Channel::Ch2, duty_ring(50, max));
Timer::after(Duration::from_millis(1000)).await;


let mut uart = Uart::new_blocking(
p.LPUART1,
p.PA3,
p.PA2,
UartConfig::default(),
    ).unwrap();

 
let i2c = I2c::new_blocking(p.I2C4, p.PB6, p.PB7, {
    let mut c = embassy_stm32::i2c::Config::default();
    c.scl_pullup = true;
    c.sda_pullup = true;
    c
});
let interface = I2CDisplayInterface::new(i2c);
let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();
display.init().unwrap();
let text_style = MonoTextStyleBuilder::new()
    .font(&FONT_9X18_BOLD)
    .text_color(BinaryColor::On)
    .build();
display.clear_buffer();
Text::with_baseline("READY", Point::new(25, 24), text_style, Baseline::Top)
    .draw(&mut display).unwrap();
display.flush().unwrap();



let mut byte: [u8; 1] = [0; 1];
let mut cmd:  [u8; 16] = [0; 16];
let mut cmd_len: usize = 0;
loop {
if uart.blocking_read(&mut byte).is_ok() {
match byte[0] {
b'\n' => {
match &cmd[..cmd_len] {
b"ROCK"     => show_rock(&mut pwm, max).await,
b"PAPER"    => show_paper(&mut pwm, max).await,
b"SCISSORS" => show_scissors(&mut pwm, max).await,


b"WIN" => {
    display.clear_buffer();
    Text::with_baseline("YOU WIN!", Point::new(10, 24), text_style, Baseline::Top)
        .draw(&mut display).unwrap();
    display.flush().unwrap();
}
b"LOSE" => {
    display.clear_buffer();
    Text::with_baseline("YOU LOSE!", Point::new(5, 24), text_style, Baseline::Top)
        .draw(&mut display).unwrap();
    display.flush().unwrap();
}
b"DRAW" => {
    display.clear_buffer();
    Text::with_baseline("DRAW!", Point::new(30, 24), text_style, Baseline::Top)
        .draw(&mut display).unwrap();
    display.flush().unwrap();
} 
                        _           => {}
                    }
cmd_len = 0;
                }
b'\r' => {}
b => {
if cmd_len < 15 {
cmd[cmd_len] = b;
cmd_len += 1;
                    }
                }
            }
        }
    }
} 