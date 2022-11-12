#[macro_export]
macro_rules! rustc_cfg {
    ($var:literal, $($cfg:tt)*) => {
        println!(concat!("cargo:rustc-cfg=", $var), $($cfg)*);
    };
}
