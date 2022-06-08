use crate::RbConfig;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Version(u32, u32);

impl Version {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self(major, minor)
    }

    pub fn major(&self) -> u32 {
        self.0
    }

    pub fn minor(&self) -> u32 {
        self.1
    }

    pub fn current(rbconfig: &RbConfig) -> Version {
        Self(
            rbconfig.get("MAJOR").parse::<i32>().unwrap() as _,
            rbconfig.get("MINOR").parse::<i32>().unwrap() as _,
        )
    }
}
