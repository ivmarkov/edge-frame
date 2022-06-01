const MAX_ASSETS: usize = 10;

#[cfg(feature = "assets-serve")]
pub mod serve {
    use core::result::Result;

    // TODO: Remove
    extern crate alloc;
    use alloc::format;

    use embedded_svc::http::server::registry::Registry;
    use embedded_svc::http::server::Response;
    use embedded_svc::http::SendHeaders;

    use log::info;

    pub type Asset = (&'static str, &'static [u8]);

    pub type Assets = [Asset; super::MAX_ASSETS];

    #[macro_export]
    macro_rules! assets {
        ($module:literal) => {
            [
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_0")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_0"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_1")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_1"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_2")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_2"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_3")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_3"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_4")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_4"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_5")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_5"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_6")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_6"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_7")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_7"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_8")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_8"))),
                ),
                (
                    env!(concat!($module, "_EDGE_FRAME_ASSET_NAME_9")),
                    include_bytes!(env!(concat!($module, "_EDGE_FRAME_ASSET_DATA_9"))),
                ),
            ]
        };
    }

    pub fn register<R>(httpd: &mut R, assets: &[Asset]) -> Result<(), R::Error>
    where
        R: Registry,
    {
        for (name, data) in assets {
            if !name.is_empty() && !data.is_empty() {
                register_asset(httpd, AssetMetadata::derive(name), data)?;
            }
        }

        Ok(())
    }

    pub fn register_asset<R>(
        httpd: &mut R,
        asset_metadata: AssetMetadata<'static>,
        data: &'static [u8],
    ) -> Result<(), R::Error>
    where
        R: Registry,
    {
        {
            let asset_metadata = asset_metadata.clone();

            httpd
                .at(&format!("/{}", asset_metadata.name))
                .inline()
                .get(move |req, mut resp| {
                    if let Some(cache_control) = &asset_metadata.cache_control {
                        resp.set_header("Cache-Control", cache_control);
                    }

                    if let Some(content_encoding) = &asset_metadata.content_encoding {
                        resp.set_header("Content-Encoding", content_encoding);
                    }

                    if let Some(content_type) = &asset_metadata.content_type {
                        resp.set_header("Content-Type", content_type);
                    }

                    resp.send_bytes(req, data)
                })?;
        }

        info!("Registered asset {:?}", asset_metadata);

        Ok(())
    }

    #[derive(Debug, Clone)]
    pub struct AssetMetadata<'a> {
        pub name: &'a str,
        pub cache_control: Option<&'a str>,
        pub content_encoding: Option<&'a str>,
        pub content_type: Option<&'a str>,
    }

    impl<'a> AssetMetadata<'a> {
        pub fn derive(name: &str) -> AssetMetadata<'_> {
            let mut split = name.split('.');

            let suffix = split.next_back().unwrap_or("");

            let (name, content_encoding) = if suffix.eq_ignore_ascii_case("gz") {
                (&name[..name.len() - 3], Some("gzip"))
            } else {
                (name, None)
            };

            let (name, cache_control) = if name.eq_ignore_ascii_case("index.html") {
                ("", "no-store")
            } else {
                (name, "public, max-age=31536000")
            };

            let suffix = if content_encoding.is_some() {
                split.next_back().unwrap_or("")
            } else {
                suffix
            };

            let content_type = if suffix.eq_ignore_ascii_case("html") {
                Some("text/html")
            } else if suffix.eq_ignore_ascii_case("css") {
                Some("text/css")
            } else if suffix.eq_ignore_ascii_case("js") {
                Some("text/javascript")
            } else if suffix.eq_ignore_ascii_case("wasm") {
                Some("application/wasm")
            } else {
                None
            };

            AssetMetadata {
                name,
                cache_control: Some(cache_control),
                content_encoding: content_encoding,
                content_type: content_type,
            }
        }
    }
}

#[cfg(feature = "assets-prepare")]
pub mod prepare {
    use std::{
        env, fs, io,
        iter::repeat,
        path::{Path, PathBuf},
    };

    use anyhow;
    use flate2::{write::GzEncoder, Compression};

    pub fn run(module: impl AsRef<str>, assets_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let module = module.as_ref();
        let assets_dir = assets_dir.as_ref();

        let output_dir = PathBuf::new()
            .join(env::var_os("OUT_DIR")
                .ok_or_else(|| anyhow::anyhow!("OUT_DIR variable is not defined. You should call this code from a Cargo `build.rs` script"))?)
            .join("edge_frame_assets")
            .join(module);

        let output_files = compress(assets_dir, &output_dir, |path| {
            println!("cargo:rerun-if-changed={}", path.display())
        })?;

        let output_files = if output_files.len() > super::MAX_ASSETS {
            anyhow::bail!(
                "Maximum number of supported assets is {}",
                super::MAX_ASSETS
            );
        } else if output_files.len() < super::MAX_ASSETS {
            let empty_file = output_dir.join("__empty__");

            fs::File::create(&empty_file)?;

            let len = output_files.len();

            output_files
                .into_iter()
                .chain(repeat(empty_file).take(super::MAX_ASSETS - len))
                .collect::<Vec<_>>()
        } else {
            output_files
        };

        for (index, output_file) in output_files.iter().enumerate() {
            println!(
                "cargo:rustc-env={}_EDGE_FRAME_ASSET_NAME_{}={}",
                module,
                index,
                output_file.file_name().unwrap().to_str().unwrap()
            );
            println!(
                "cargo:rustc-env={}_EDGE_FRAME_ASSET_DATA_{}={}",
                module,
                index,
                output_file.display()
            );
        }

        Ok(())
    }

    pub fn compress(
        assets_dir: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        track: impl Fn(&Path),
    ) -> anyhow::Result<Vec<PathBuf>> {
        let assets_dir = assets_dir.as_ref();
        let output_dir = output_dir.as_ref();

        let output_files = fs::read_dir(assets_dir)?
            .filter_map(|file| file.ok())
            .filter(|file| file.metadata().map(|md| md.is_file()).unwrap_or(false))
            .map(|file| {
                track(&file.path());

                let output_file =
                    output_dir.join(format!("{}.gz", file.file_name().to_str().unwrap()));

                track(&output_file);

                fs::create_dir_all(output_dir).unwrap();

                io::copy(
                    &mut fs::File::open(file.path()).unwrap(),
                    &mut GzEncoder::new(
                        fs::File::create(&output_file).unwrap(),
                        Compression::best(),
                    ),
                )
                .unwrap();

                output_file
            })
            .collect::<Vec<_>>();

        Ok(output_files)
    }
}
