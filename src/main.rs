#![feature(clamp)]

use anyhow::{Context, Result};
use ini::Ini;
use nl80211::{parse_hex, parse_u8, Socket, Station};
use std::collections::HashMap;
use std::env;
use std::{thread, time};
use sysfs_class::{Leds, SysClass};

mod utils {
    pub mod openwrt_leds;
}

use crate::utils::openwrt_leds::Trigger;

#[derive(Default, Clone)]
pub struct Config {
    dump_delay: Option<u64>,
    low_signal_cap: Option<u64>,
    no_signal_trigger: Option<String>,
    no_signal_delay: Option<u64>,
    signal_trigger: Option<String>,
    signal_delay: Option<u64>,
    low_signal_trigger: Option<String>,
    low_signal_delay: Option<u64>,
}

impl Config {
    fn parse(&mut self, k: &str, v: &str) -> Config {
        match k {
            "dump_delay" => {
                self.dump_delay = Some(v.parse::<u64>().unwrap());
            }
            "low_signal_cap" => {
                self.low_signal_cap = Some(v.parse::<u64>().unwrap());
            }
            "no_signal_trigger" => {
                self.no_signal_trigger = Some(v.to_string());
            }
            "no_signal_delay" => {
                self.no_signal_delay = Some(v.parse::<u64>().unwrap());
            }
            "signal_trigger" => {
                self.signal_trigger = Some(v.to_string());
            }
            "signal_delay" => {
                self.signal_delay = Some(v.parse::<u64>().unwrap());
            }
            "low_signal_trigger" => {
                self.low_signal_trigger = Some(v.to_string());
            }
            "low_signal_delay" => {
                self.low_signal_delay = Some(v.parse::<u64>().unwrap());
            }
            _ => panic!("Invalid key {}", k),
        }

        self.to_owned()
    }
}

#[derive(Default, Clone)]
pub struct Client {
    bssid: Option<String>,
    led: Option<Leds>,
    low_signal_cap: Option<u64>,
    no_signal_trigger: Option<String>,
    no_signal_delay: Option<u64>,
    signal_trigger: Option<String>,
    signal_delay: Option<u64>,
    low_signal_trigger: Option<String>,
    low_signal_delay: Option<u64>,
}

impl Client {
    fn parse(&mut self, k: &str, v: &str) -> Client {
        match k {
            "bssid" => {
                self.bssid = Some(v.to_string());
            }
            "led" => {
                let path = Leds::dir().join(v.to_string());
                self.led = Some(Leds::from_path(path.as_path()).unwrap());
            }
            "low_signal_cap" => {
                self.low_signal_cap = Some(v.parse::<u64>().unwrap());
            }
            "no_signal_trigger" => {
                self.no_signal_trigger = Some(v.to_string());
            }
            "no_signal_delay" => {
                self.no_signal_delay = Some(v.parse::<u64>().unwrap());
            }
            "signal_trigger" => {
                self.signal_trigger = Some(v.to_string());
            }
            "signal_delay" => {
                self.signal_delay = Some(v.parse::<u64>().unwrap());
            }
            "low_signal_trigger" => {
                self.low_signal_trigger = Some(v.to_string());
            }
            "low_signal_delay" => {
                self.low_signal_delay = Some(v.parse::<u64>().unwrap());
            }
            _ => panic!("Invalid key {}", k),
        }

        self.to_owned()
    }
}

fn bss_dump(config: &Config, clients: &Vec<Client>) -> Result<()> {
    let mut cstations: HashMap<String, Station> = HashMap::with_capacity(255);

    let interfaces = Socket::connect()?.get_interfaces_info()?;
    for interface in interfaces {
        let stations = interface.get_stations_info();
        for station in stations? {
            cstations.insert(
                parse_hex(station.bssid.as_ref().context("Invalid bssid")?),
                station,
            );
        }
    }

    for client in clients {
        let bssid = client.bssid.as_ref().context("Failed to get bssid")?;
        match cstations.get(bssid) {
            Some(station) => {
                let led = client.led.as_ref().context("Failed to get led object")?;
                let signal =
                    100 + (parse_u8(&station.signal.clone().context("Failed to get signal")?) as u64);
                if signal
                    > client.low_signal_cap.unwrap_or(
                        config
                            .low_signal_cap
                            .context("Failed to get low_signal_cap")?,
                    )
                {
                    let trigger = client.signal_trigger.as_ref().unwrap_or(
                        config
                            .signal_trigger
                            .as_ref()
                            .context("Failed to get signal_trigger")?,
                    );
                    led.set_trigger(trigger)?;
                    if trigger == "timer" {
                        let delay = client
                            .signal_delay
                            .unwrap_or(config.signal_delay.context("Failed to get signal_delay")?);
                        led.set_delay_on(delay*signal)?;
                        led.set_delay_off(delay*signal)?;
                    }
                } else {
                    let trigger = client.low_signal_trigger.as_ref().unwrap_or(
                        config
                            .low_signal_trigger
                            .as_ref()
                            .context("Failed to get signal_trigger")?,
                    );
                    led.set_trigger(trigger)?;
                    if trigger == "timer" {
                        let delay = client.low_signal_delay.unwrap_or(
                            config
                                .low_signal_delay
                                .context("Failed to get signal_delay")?,
                        );
                        led.set_delay_on(delay * signal)?;
                        led.set_delay_off(delay * signal)?;
                    }
                }
            }
            None => {
                let led = client.led.as_ref().context("Failed to get led object")?;
                let trigger = client.no_signal_trigger.as_ref().unwrap_or(
                    config
                        .no_signal_trigger
                        .as_ref()
                        .context("Failed to get signal_trigger")?,
                );
                led.set_trigger(trigger)?;
                if trigger == "timer" {
                    let delay = client.no_signal_delay.unwrap_or(
                        config
                            .no_signal_delay
                            .context("Failed to get signal_delay")?,
                    );
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
            for (k, v) in prop.iter() {
                config = config.parse(k, v);
            }
        } else {
            let mut client = Client::default();
            for (k, v) in prop.iter() {
                client = client.parse(k, v);
            }
            clients.push(client);
        }
    }

    println!("Loaded config from {}",path);

    let dump_delay =
        time::Duration::from_secs(config.dump_delay.context("Dump delay isn't specified")?);

        println!("{:#?}",dump_delay);

    loop {
        bss_dump(&config, &clients)?;
        thread::sleep(dump_delay);
    }
}
