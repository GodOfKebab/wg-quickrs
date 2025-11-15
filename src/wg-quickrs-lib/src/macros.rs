// src/version.rs
include!(concat!(env!("OUT_DIR"), "/version_macro.rs"));

#[allow(unused_imports)]
pub use wg_quickrs_version;
#[allow(unused_imports)]
pub use build_git_branch_name;
#[allow(unused_imports)]
pub use build_git_commit;
#[allow(unused_imports)]
pub use build_timestamp;
#[allow(unused_imports)]
pub use full_version;
