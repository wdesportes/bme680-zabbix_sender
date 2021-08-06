use bme680::*;
use core::result;
use core::time::Duration;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c;
use linux_embedded_hal as hal;
use linux_embedded_hal::Delay;
use log::error;
use log::info;
use std::env;
use zbx_sender::{Response, Result, SendValue, Sender};

fn send(data: &FieldData) -> Result<Response> {
    let zabbix_host: String = env::var("ZABBIX_HOST").expect("Env ZABBIX_HOST must be set");
    let zabbix_send_as_host: String =
        env::var("ZABBIX_SEND_AS_HOST").expect("Env ZABBIX_SEND_AS_HOST must be set");
    let zabbix_port: String = env::var("ZABBIX_PORT").unwrap_or("10051".to_string());
    let sender = Sender::new(zabbix_host, zabbix_port.parse::<u16>().unwrap());
    let collection: Vec<SendValue> = [
        (
            zabbix_send_as_host.as_ref(),
            "bme680.temperature",
            format!("{}", data.temperature_celsius()).as_ref(),
        )
            .into(),
        (
            zabbix_send_as_host.as_ref(),
            "bme680.pressure",
            format!("{}", data.pressure_hpa()).as_ref(),
        )
            .into(),
        (
            zabbix_send_as_host.as_ref(),
            "bme680.humidity",
            format!("{}", data.humidity_percent()).as_ref(),
        )
            .into(),
        (
            zabbix_send_as_host.as_ref(),
            "bme680.gas-resistence",
            format!("{}", data.gas_resistance_ohm()).as_ref(),
        )
            .into(),
    ]
    .iter()
    .cloned()
    .collect();

    sender.send(collection)
}

fn main(
) -> result::Result<(), Error<<hal::I2cdev as i2c::Read>::Error, <hal::I2cdev as i2c::Write>::Error>>
{
    env_logger::init();
    let i2c = hal::I2cdev::new("/dev/i2c-1").unwrap();
    let mut delayer = Delay {};
    let mut dev = Bme680::init(i2c, &mut delayer, I2CAddress::Primary)?;
    let mut delay = Delay {};

    let settings = SettingsBuilder::new()
        .with_humidity_oversampling(OversamplingSetting::OS2x)
        .with_pressure_oversampling(OversamplingSetting::OS4x)
        .with_temperature_oversampling(OversamplingSetting::OS8x)
        .with_temperature_filter(IIRFilterSize::Size3)
        .with_gas_measurement(Duration::from_millis(1500), 320, 25)
        .with_temperature_offset(-2.2)
        .with_run_gas(true)
        .build();

    let profile_dur = dev.get_profile_dur(&settings.0)?;
    info!("Profile duration {:?}", profile_dur);
    info!("Setting sensor settings");
    dev.set_sensor_settings(&mut delayer, settings)?;
    info!("Setting forced power modes");
    dev.set_sensor_mode(&mut delayer, PowerMode::ForcedMode)?;

    let sensor_settings = dev.get_sensor_settings(settings.1);
    info!("Sensor settings: {:?}", sensor_settings);

    let wait_time_string: String = env::var("WAIT_TIME_MS").unwrap_or("5000".to_string());
    let wait_time: u32 = wait_time_string.parse::<u32>().unwrap();

    loop {
        delay.delay_ms(wait_time);
        let power_mode = dev.get_sensor_mode();
        info!("Sensor power mode: {:?}", power_mode);
        info!("Setting forced power modes");
        dev.set_sensor_mode(&mut delayer, PowerMode::ForcedMode)?;
        info!("Retrieving sensor data");
        let (data, _state) = dev.get_sensor_data(&mut delayer)?;
        info!("Sensor Data {:?}", data);
        match send(&data) {
            Ok(response) => {
                info!("{:?}", response);
                info!(
                    "processed: {}; failed: {}; total: {}; seconds spent: {}",
                    response.processed_cnt().unwrap(),
                    response.failed_cnt().unwrap(),
                    response.total_cnt().unwrap(),
                    response.seconds_spent().unwrap()
                );
            }
            Err(e) => error!("Error {}", e),
        }
        info!("Temperature {}°C", data.temperature_celsius());
        info!("Pressure {}hPa", data.pressure_hpa());
        info!("Humidity {}%", data.humidity_percent());
        info!("Gas Resistence {}Ω", data.gas_resistance_ohm());
    }
}
