// SPDX-License-Identifier: Apache-2.0
//! Application storage paths; no automatic migration of legacy wallets.
use std::path::PathBuf;

pub fn default_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var_os("APPDATA").map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData/Roaming")))
            .expect("APPDATA or USERPROFILE is required for the wallet data directory");
        root.join("RetroBoomer")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(std::env::var_os("HOME").expect("HOME is required for the wallet data directory"))
            .join(".retro-boomer")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn platform_default_has_the_approved_name() {
        let path = super::default_data_dir();
        assert!(path.is_absolute());
        assert_eq!(path.file_name().unwrap(), if cfg!(windows) { "RetroBoomer" } else { ".retro-boomer" });
    }
}
