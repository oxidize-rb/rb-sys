#[macro_export]
macro_rules! rustc_cfg {
    ($enable:expr, $var:literal, $($cfg:tt)*) => {
        println!(concat!("cargo:rustc-check-cfg=cfg(", $var, ")"), $($cfg)*);
        if $enable {
            println!(concat!("cargo:rustc-cfg=", $var), $($cfg)*);
        }
    };
}
