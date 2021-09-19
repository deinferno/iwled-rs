#![feature(clamp)]

#[macro_use]
extern crate derive_new;

mod config;
use config::*;

mod leds;
use leds::Trigger;

use anyhow::Result;
use ini::Ini;
use nl80211::{parse_hex, parse_u8, Socket, Station};
use std::collections::HashMap;
use std::env;
use std::{thread, time};

fn bss_dump(config: &Config, clients: &Vec<Client>) -> Result<()> {
    let interfaces = Socket::connect()?.get_interfaces_info()?;

    let cstations: HashMap<String, Station> = interfaces
        .iter()
        .flat_map(|int| int.get_stations_info())
        .flatten()
        .filter(|s| s.bssid.is_some())
        .map(|s| (parse_hex(s.bssid.as_ref().unwrap()), s))
        .collect();

    for client in clients {
        let led = &client.led;
        match cstations.get(&client.bssid) {
            Some(station) => {
                let signal = station
                    .signal
                    .as_ref()
                    .map(|s| 100 + (parse_u8(&s) as u64))
                    .unwrap_or_default();

                if signal > client.low_signal_cap.unwrap_or(config.low_signal_cap) {
                    let trigger = client
                        .signal_trigger
                        .as_ref()
                        .unwrap_or(&config.signal_trigger);
                    led.set_trigger(trigger)?;

                    if trigger == "timer" {
                        let delay = client.signal_delay.unwrap_or(config.signal_delay);
                        led.set_delay_on(delay * signal)?;
                        led.set_delay_off(delay * signal)?;
                    }
                } else {
                    let trigger = client
                        .low_signal_trigger
                        .as_ref()
                        .unwrap_or(&config.low_signal_trigger);

                    led.set_trigger(trigger)?;

                    if trigger == "timer" {
                        let delay = client.low_signal_delay.unwrap_or(config.low_signal_delay);
                        led.set_delay_on(delay * signal)?;
                        led.set_delay_off(delay * signal)?;
                    }
                }
            }
            None => {
                let trigger = client
                    .no_signal_trigger
                    .as_ref()
                    .unwrap_or(&config.no_signal_trigger);

                led.set_trigger(trigger)?;
                if trigger == "timer" {
                    let delay = client.no_signal_delay.unwrap_or(config.no_signal_delay);
                    led.set_delay_on(delay)?;
                    led.set_delay_off(delay)?;
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("Starting iwled-rs");

    let path = env::args()
        .nth(1)
        .unwrap_or("/etc/iwled-rs/iwled-rs.conf".to_string());

    let ini = Ini::load_from_file(&path)?;

    let mut config = Config::default();
    let mut clients: Vec<Client> = Vec::with_capacity(255);

    for (sec, prop) in ini.iter() {
        if sec.is_none() {
            config = Config::from_properties(prop)?;
        } else {
            clients.push(Client::from_properties(prop)?);
        }
    }

    println!("Loaded config from {}", path);

    loop {
        bss_dump(&config, &clients)?;
        thread::sleep(time::Duration::from_secs(config.dump_delay));
    }
}
