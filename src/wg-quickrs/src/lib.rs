use std::path::PathBuf;
use once_cell::sync::OnceCell;

pub static WG_QUICKRS_CONFIG_FOLDER: OnceCell<PathBuf> = OnceCell::new();
pub static WG_QUICKRS_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();
pub static WIREGUARD_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

pub mod commands;
pub mod conf;
pub mod web;
pub mod wireguard;
pub mod macros;
