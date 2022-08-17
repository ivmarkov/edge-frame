#![allow(clippy::let_unit_value)]
#![cfg_attr(
    any(feature = "assets-serve", all(feature = "dto", not(feature = "web"))),
    no_std
)]
#![cfg_attr(
    all(feature = "nightly", feature = "assets-serve"),
    feature(generic_associated_types)
)]
#![cfg_attr(
    all(feature = "nightly", feature = "assets-serve"),
    feature(type_alias_impl_trait)
)]
#![cfg_attr(feature = "web", recursion_limit = "1024")]

#[cfg(any(
    all(feature = "assets-prepare", feature = "assets-serve"),
    all(feature = "assets-prepare", feature = "web"),
    all(feature = "assets-prepare", feature = "dto")
))]
compile_error!(
    "Feature `assets-prepare` is not compatible with features `assets-serve`, `web` and `dto`."
);

#[cfg(all(feature = "assets-serve", feature = "web"))]
compile_error!("Feature `assets-serve` is not compatible with feature `web`.");

#[cfg(feature = "web")]
pub use web::*;

#[cfg(feature = "web")]
#[path = "."]
mod web {
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

#[cfg(any(feature = "assets-serve", feature = "assets-prepare"))]
pub mod assets;

#[cfg(feature = "dto")]
pub mod dto;
