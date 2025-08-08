use std::sync::LazyLock;

use semver::Version;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub fn get_ver() -> &'static Version {
    &VERSION
}

fn init_ver() -> Version {
    lenient_semver::parse(VERSION_FULL).expect("Expected a valid version at compile time.")
}

pub static VERSION: LazyLock<Version> = LazyLock::new(init_ver);
