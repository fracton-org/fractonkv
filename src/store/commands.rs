use std::fmt::Display;
use strum::{Display, EnumString};

#[derive(EnumString, Display, Debug)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Command {
    Get,
    Set,
}

impl Command {
    fn route() {}
}
