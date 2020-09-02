use std::io::Result;
use sysfs_class::{set_trait_method, Leds, SysClass};

pub trait Trigger: SysClass {
    set_trait_method!("trigger", set_trigger);
    set_trait_method!("delay_on", set_delay_on u64);
    set_trait_method!("delay_off", set_delay_off u64);
}

impl Trigger for Leds {}
