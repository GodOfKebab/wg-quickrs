// src/version.rs
include!(concat!(env!("OUT_DIR"), "/version_macro.rs"));

#[allow(unused_imports)]
pub use build_info;
#[allow(unused_imports)]
pub use full_version;
#[allow(unused_imports)]
pub use wg_quickrs_version;
