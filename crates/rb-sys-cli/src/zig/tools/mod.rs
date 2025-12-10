//! Tool-specific implementations of the ZigShim trait.

pub mod ar;
pub mod cc;
pub mod dlltool;
pub mod ld;

pub use ar::{ZigAr, ZigArArgs};
pub use cc::{ZigCc, ZigCcArgs};
pub use dlltool::{ZigDlltool, ZigDlltoolArgs};
pub use ld::{ZigLd, ZigLdArgs};
