#![cfg_attr(feature = "assets-register", no_std)]
#![cfg_attr(feature = "web", recursion_limit = "1024")]

#[cfg(any(
    all(feature = "assets-register", feature = "assets-prepare"),
    all(
        feature = "web",
        any(feature = "assets-register", feature = "assets-prepare")
    ),
    not(any(
        feature = "web",
        feature = "assets-register",
        feature = "assets-prepare"
    ))
))]
compile_error!(
    "Exactly one of the features `web', `assets-register` or `assets-prepare` must be selected."
);

#[cfg(not(any(feature = "assets-register", feature = "assets-prepare")))]
pub use main::*;

#[cfg(not(any(feature = "assets-register", feature = "assets-prepare")))]
#[path = "."]
mod main {
    pub mod auth;
    pub mod callback2;
    pub mod field;
    pub mod frame;
    pub mod loading;
    pub mod redust;
    pub mod role;
    pub mod util;
    pub mod wifi;
}

#[cfg(any(feature = "assets-register", feature = "assets-prepare"))]
pub mod assets;
