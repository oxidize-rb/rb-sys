#[cfg(feature = "local")]
mod local;
#[cfg(feature = "local")]
use local::run;

// #[cfg(not(feature = "local"))]
mod prebuilt;
#[cfg(not(feature = "local"))]
use prebuilt::run;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}
