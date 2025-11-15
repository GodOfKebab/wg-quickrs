use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use crate::macros::*;

#[derive(Error, Debug)]
pub enum WireGuardLibError {
    #[error("peer {0} is not found")]
    PeerNotFound(Uuid),
    #[error("unable to serialize network")]
    SerializationFailed(),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum WireGuardStatus {
    UNKNOWN,
    DOWN,
    UP,
}

#[derive(Serialize, Deserialize)]
pub struct VersionBuildInfo {
    pub version: &'static str,
    pub build: BuildInfo,
}

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    pub branch: &'static str,
    pub commit: &'static str,
    pub timestamp: &'static str,
}

pub const VERSION_BUILD_INFO: VersionBuildInfo = VersionBuildInfo {
    version: wg_quickrs_version!(),
    build: BuildInfo {
        branch: build_git_branch_name!(),
        commit: build_git_commit!(),
        timestamp: build_timestamp!(),
    },
};
