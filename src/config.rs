use anyhow::{Context, Result};
use ini::Properties;
use sysfs_class::{Leds, SysClass};

trait PropertiesResult {
    fn get_result<S: AsRef<str> + std::fmt::Display>(&self, s: S) -> Result<&str>;
}

impl PropertiesResult for Properties {
    fn get_result<S: AsRef<str> + std::fmt::Display>(&self, s: S) -> Result<&str> {
        self.get(&s).context(format!("{} is missing", &s))
    }
}

#[derive(Default, Clone, new)]
pub struct Config {
    pub dump_delay: u64,
    pub low_signal_cap: u64,
    pub no_signal_trigger: String,
    pub no_signal_delay: u64,
    pub signal_trigger: String,
    pub signal_delay: u64,
    pub low_signal_trigger: String,
    pub low_signal_delay: u64,
}

impl Config {
    pub fn from_properties(prop: &Properties) -> Result<Config> {
        Ok(Config::new(
            prop.get_result("dump_delay")?.parse()?,
            prop.get_result("low_signal_cap")?.parse()?,
            prop.get_result("no_signal_trigger")?.parse()?,
            prop.get_result("no_signal_delay")?.parse()?,
            prop.get_result("signal_trigger")?.parse()?,
            prop.get_result("signal_delay")?.parse()?,
            prop.get_result("low_signal_trigger")?.parse()?,
            prop.get_result("low_signal_delay")?.parse()?,
        ))
    }
}

#[derive(Clone, new)]
pub struct Client {
    pub bssid: String,
    pub led: Leds,
    pub low_signal_cap: Option<u64>,
    pub no_signal_trigger: Option<String>,
    pub no_signal_delay: Option<u64>,
    pub signal_trigger: Option<String>,
    pub signal_delay: Option<u64>,
    pub low_signal_trigger: Option<String>,
    pub low_signal_delay: Option<u64>,
}

impl Client {
    pub fn from_properties(prop: &Properties) -> Result<Client> {
        Ok(Client::new(
            prop.get_result("bssid")?.parse()?,
            Leds::from_path(
                Leds::dir()
                    .join(prop.get_result("led")?.parse::<String>()?)
                    .as_path(),
            )?,
            prop.get("low_signal_cap").and_then(|s| s.parse().ok()),
            prop.get("no_signal_trigger").and_then(|s| s.parse().ok()),
            prop.get("no_signal_delay").and_then(|s| s.parse().ok()),
            prop.get("signal_trigger").and_then(|s| s.parse().ok()),
            prop.get("signal_delay").and_then(|s| s.parse().ok()),
            prop.get("low_signal_trigger").and_then(|s| s.parse().ok()),
            prop.get("low_signal_delay").and_then(|s| s.parse().ok()),
        ))
    }
}
