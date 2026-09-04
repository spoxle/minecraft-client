use super::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Platform {
    runtime_key: &'static str,
    rule_os: &'static str,
    rule_arch: &'static str,
    native_arch: &'static str,
}

impl Platform {
    pub fn current() -> Result<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => Ok(Self::new("windows-x64", "windows", "x86_64", "64")),
            ("windows", "x86") => Ok(Self::new("windows-x86", "windows", "x86", "32")),
            ("windows", "aarch64") => Ok(Self::new("windows-arm64", "windows", "arm64", "arm64")),
            ("macos", "x86_64") => Ok(Self::new("mac-os", "osx", "x86_64", "64")),
            ("macos", "aarch64") => Ok(Self::new("mac-os-arm64", "osx", "arm64", "arm64")),
            ("linux", "x86_64") => Ok(Self::new("linux", "linux", "x86_64", "64")),
            ("linux", "x86") => Ok(Self::new("linux-i386", "linux", "x86", "32")),
            _ => Err(Error::UnsupportedPlatform),
        }
    }

    pub const fn new(
        runtime_key: &'static str,
        rule_os: &'static str,
        rule_arch: &'static str,
        native_arch: &'static str,
    ) -> Self {
        Self {
            runtime_key,
            rule_os,
            rule_arch,
            native_arch,
        }
    }

    pub fn runtime_key(self) -> &'static str {
        self.runtime_key
    }
    pub fn rule_os(self) -> &'static str {
        self.rule_os
    }
    pub fn rule_arch(self) -> &'static str {
        self.rule_arch
    }
    pub fn native_arch(self) -> &'static str {
        self.native_arch
    }
}
