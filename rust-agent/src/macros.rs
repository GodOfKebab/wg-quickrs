// src/version.rs
include!(concat!(env!("OUT_DIR"), "/version_macro.rs"));

#[allow(unused_imports)]
pub use backend_version;
#[allow(unused_imports)]
pub use build_info;
#[allow(unused_imports)]
pub use frontend_version;
#[allow(unused_imports)]
pub use full_version;
