#![cfg(feature = "pi")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rppal::gpio::{Gpio, InputPin, Trigger};
use rppal::pwm::{Channel, Polarity, Pwm};

// Default pin/frequency settings; adjust as needed
const PWM_PIN: u8 = 12; // GPIO12 supports PWM0
const TACH_PIN: u8 = 18; // Example tach pin
const PWM_FREQ_HZ: f64 = 1000.0;
const DUTY_PERCENT: f64 = 100.0;
const DHT_PIN: u8 = 26; // DHT22 data pin

pub struct PiSensors {
    pwm: Pwm,
    _tach_pin: InputPin, // kept to hold interrupt registration
    pulse_count: Arc<AtomicUsize>,
}

impl PiSensors {
    pub fn new() -> anyhow::Result<Self> {
        // Initialize PWM on PWM0 (GPIO12)
        let pwm = Pwm::with_frequency(
            Channel::Pwm0,
            PWM_FREQ_HZ,
            DUTY_PERCENT / 100.0,
            Polarity::Normal,
            true,
        )?;

        // Setup tach input with rising edge interrupt
        let gpio = Gpio::new()?;
        let mut tach = gpio.get(TACH_PIN)?.into_input_pulldown();

        let pulse_count = Arc::new(AtomicUsize::new(0));
        let pc = pulse_count.clone();

        tach.set_async_interrupt(Trigger::RisingEdge, move |_| {
            pc.fetch_add(1, Ordering::Relaxed);
        })?;

        Ok(Self {
            pwm,
            _tach_pin: tach,
            pulse_count,
        })
    }

    pub fn set_pwm_duty_percent(&mut self, duty_percent: f64) -> anyhow::Result<()> {
        let duty = (duty_percent.max(0.0).min(100.0)) / 100.0;
        self.pwm.set_duty_cycle(duty)?;
        Ok(())
    }

    // Returns pulses counted since last call and resets counter
    pub fn take_pulses(&self) -> usize {
        self.pulse_count.swap(0, Ordering::Relaxed)
    }

    pub fn pwm_duty_percent(&self) -> anyhow::Result<f64> {
        Ok(self.pwm.duty_cycle()? * 100.0)
    }
}

pub fn read_dht22() -> Option<(f32, f32)> {
    // TODO: Implement using dht_sensor::dht22::Reading::read with an embedded-hal delay and pin adapter.
    // Placeholder to avoid build error until adapter is added.
    None
}


