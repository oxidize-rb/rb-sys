// Auto-generated from data/toolchains.json
// DO NOT EDIT MANUALLY

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toolchain {
    ArmLinux,
    Aarch64Linux,
    Aarch64LinuxMusl,
    Arm64Darwin,
    X64MingwUcrt,
    Aarch64MingwUcrt,
    X64Mingw32,
    X86Linux,
    X86Mingw32,
    X8664Darwin,
    X8664Linux,
    X8664LinuxMusl,
}

impl Toolchain {
    pub const fn ruby_platform(&self) -> &'static str {
        match self {
            Toolchain::ArmLinux => "arm-linux",
            Toolchain::Aarch64Linux => "aarch64-linux",
            Toolchain::Aarch64LinuxMusl => "aarch64-linux-musl",
            Toolchain::Arm64Darwin => "arm64-darwin",
            Toolchain::X64MingwUcrt => "x64-mingw-ucrt",
            Toolchain::Aarch64MingwUcrt => "aarch64-mingw-ucrt",
            Toolchain::X64Mingw32 => "x64-mingw32",
            Toolchain::X86Linux => "x86-linux",
            Toolchain::X86Mingw32 => "x86-mingw32",
            Toolchain::X8664Darwin => "x86_64-darwin",
            Toolchain::X8664Linux => "x86_64-linux",
            Toolchain::X8664LinuxMusl => "x86_64-linux-musl",
        }
    }

    pub const fn rust_target(&self) -> &'static str {
        match self {
            Toolchain::ArmLinux => "arm-unknown-linux-gnueabihf",
            Toolchain::Aarch64Linux => "aarch64-unknown-linux-gnu",
            Toolchain::Aarch64LinuxMusl => "aarch64-unknown-linux-musl",
            Toolchain::Arm64Darwin => "aarch64-apple-darwin",
            Toolchain::X64MingwUcrt => "x86_64-pc-windows-gnu",
            Toolchain::Aarch64MingwUcrt => "aarch64-pc-windows-gnullvm",
            Toolchain::X64Mingw32 => "x86_64-pc-windows-gnu",
            Toolchain::X86Linux => "i686-unknown-linux-gnu",
            Toolchain::X86Mingw32 => "i686-pc-windows-gnu",
            Toolchain::X8664Darwin => "x86_64-apple-darwin",
            Toolchain::X8664Linux => "x86_64-unknown-linux-gnu",
            Toolchain::X8664LinuxMusl => "x86_64-unknown-linux-musl",
        }
    }

    pub const fn sysroot_paths(&self) -> &'static [&'static str] {
        match self {
            Toolchain::ArmLinux => &["/usr/include", "/usr/lib/arm-linux-gnueabihf"],
            Toolchain::Aarch64Linux => &["/usr/include", "/usr/lib/aarch64-linux-gnu"],
            Toolchain::Aarch64LinuxMusl => &["/usr/include", "/usr/lib/aarch64-linux-musl"],
            Toolchain::Arm64Darwin => &[],
            Toolchain::X64MingwUcrt => &[],
            Toolchain::Aarch64MingwUcrt => &[],
            Toolchain::X64Mingw32 => &[],
            Toolchain::X86Linux => &["/usr/include", "/usr/lib/i386-linux-gnu"],
            Toolchain::X86Mingw32 => &[],
            Toolchain::X8664Darwin => &[],
            Toolchain::X8664Linux => &["/usr/include", "/usr/lib/x86_64-linux-gnu"],
            Toolchain::X8664LinuxMusl => &["/usr/include", "/usr/lib/x86_64-linux-musl"],
        }
    }

    pub fn rake_compiler_image(&self) -> String {
        format!("ghcr.io/rake-compiler/rake-compiler-dock-image:1.10.0-mri-{}", self.ruby_platform())
    }

    pub const fn supported(&self) -> bool {
        match self {
            Toolchain::ArmLinux => true,
            Toolchain::Aarch64Linux => true,
            Toolchain::Aarch64LinuxMusl => true,
            Toolchain::Arm64Darwin => true,
            Toolchain::X64MingwUcrt => true,
            Toolchain::Aarch64MingwUcrt => true,
            Toolchain::X64Mingw32 => true,
            Toolchain::X86Linux => false,
            Toolchain::X86Mingw32 => false,
            Toolchain::X8664Darwin => true,
            Toolchain::X8664Linux => true,
            Toolchain::X8664LinuxMusl => true,
        }
    }

    pub fn from_ruby_platform(platform: &str) -> Option<Toolchain> {
        match platform {
            "arm-linux" => Some(Toolchain::ArmLinux),
            "aarch64-linux" => Some(Toolchain::Aarch64Linux),
            "aarch64-linux-musl" => Some(Toolchain::Aarch64LinuxMusl),
            "arm64-darwin" => Some(Toolchain::Arm64Darwin),
            "x64-mingw-ucrt" => Some(Toolchain::X64MingwUcrt),
            "aarch64-mingw-ucrt" => Some(Toolchain::Aarch64MingwUcrt),
            "x64-mingw32" => Some(Toolchain::X64Mingw32),
            "x86-linux" => Some(Toolchain::X86Linux),
            "x86-mingw32" => Some(Toolchain::X86Mingw32),
            "x86_64-darwin" => Some(Toolchain::X8664Darwin),
            "x86_64-linux" => Some(Toolchain::X8664Linux),
            "x86_64-linux-musl" => Some(Toolchain::X8664LinuxMusl),
            _ => None,
        }
    }

    pub fn from_rust_target(target: &str) -> Option<Toolchain> {
        match target {
            "arm-unknown-linux-gnueabihf" => Some(Toolchain::ArmLinux),
            "aarch64-unknown-linux-gnu" => Some(Toolchain::Aarch64Linux),
            "aarch64-unknown-linux-musl" => Some(Toolchain::Aarch64LinuxMusl),
            "aarch64-apple-darwin" => Some(Toolchain::Arm64Darwin),
            "x86_64-pc-windows-gnu" => Some(Toolchain::X64MingwUcrt),
            "aarch64-pc-windows-gnullvm" => Some(Toolchain::Aarch64MingwUcrt),
            "i686-unknown-linux-gnu" => Some(Toolchain::X86Linux),
            "i686-pc-windows-gnu" => Some(Toolchain::X86Mingw32),
            "x86_64-apple-darwin" => Some(Toolchain::X8664Darwin),
            "x86_64-unknown-linux-gnu" => Some(Toolchain::X8664Linux),
            "x86_64-unknown-linux-musl" => Some(Toolchain::X8664LinuxMusl),
            _ => None,
        }
    }

    pub fn all_supported() -> impl Iterator<Item = Toolchain> {
        [
            Toolchain::ArmLinux,
            Toolchain::Aarch64Linux,
            Toolchain::Aarch64LinuxMusl,
            Toolchain::Arm64Darwin,
            Toolchain::X64MingwUcrt,
            Toolchain::Aarch64MingwUcrt,
            Toolchain::X64Mingw32,
            Toolchain::X86Linux,
            Toolchain::X86Mingw32,
            Toolchain::X8664Darwin,
            Toolchain::X8664Linux,
            Toolchain::X8664LinuxMusl,
        ].into_iter().filter(|t| t.supported())
    }
}
