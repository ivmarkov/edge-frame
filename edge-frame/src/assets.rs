const MAX_ASSETS: usize = 10;

#[cfg(feature = "assets-serve")]
pub mod serve {
    use core::result::Result;

    extern crate alloc;
    use alloc::borrow::Cow;
    use alloc::format;

    use embedded_svc::http::server::registry::Registry;
    use embedded_svc::http::server::Response;
    use embedded_svc::http::SendHeaders;

    const ASSETS: [(&'static str, &'static [u8]); super::MAX_ASSETS] = [
        (
            env!("EDGE_FRAME_ASSET_0_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_0_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_1_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_1_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_2_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_2_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_3_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_3_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_4_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_4_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_5_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_5_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_6_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_6_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_7_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_7_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_8_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_8_DATA")),
        ),
        (
            env!("EDGE_FRAME_ASSET_9_NAME"),
            include_bytes!(env!("EDGE_FRAME_ASSET_9_DATA")),
        ),
    ];

    pub fn register<R>(httpd: &mut R) -> Result<(), R::Error>
    where
        R: Registry,
    {
        for (name, data) in ASSETS {
            if !name.is_empty() && !data.is_empty() {
                register_asset(httpd, name, data, ContentMetadata::derive(name))?;
            }
        }

        Ok(())
    }

    pub fn register_asset<R>(
        httpd: &mut R,
        name: &str,
        data: &'static [u8],
        content_metadata: ContentMetadata<'static>,
    ) -> Result<(), R::Error>
    where
        R: Registry,
    {
        let uri = if content_metadata.root { "" } else { name };

        httpd.at(format!("/{}", uri)).inline().get(|req, resp| {
            if let Some(cache_control) = content_metadata.cache_control {
                resp.header("Cache-Control", cache_control);
            }

            if let Some(content_encoding) = content_metadata.content_encoding {
                resp.header("Content-Encoding", content_encoding);
            }

            if let Some(content_type) = content_metadata.content_type {
                resp.header("Content-Type", content_type);
            }

            resp.send_bytes(req, data)
        })?;

        Ok(())
    }

    pub struct ContentMetadata<'a> {
        pub root: bool,
        pub cache_control: Option<Cow<'a, str>>,
        pub content_encoding: Option<Cow<'a, str>>,
        pub content_type: Option<Cow<'a, str>>,
    }

    impl<'a> ContentMetadata<'a> {
        pub fn derive(name: &str) -> ContentMetadata<'static> {
            let root = name.eq_ignore_ascii_case("index.html")
                || name.eq_ignore_ascii_case("index.html.gz");

            let cache_control = Some(if root {
                "no-store"
            } else {
                "public, max-age=31536000"
            });

            let split = name.split('.');

            let suffix = split.next_back().unwrap_or("");

            let content_encoding = if suffix.eq_ignore_ascii_case("gz") {
                Some("gzip")
            } else {
                None
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

            ContentMetadata {
                root,
                cache_control: cache_control.map(Cow::Borrowed),
                content_encoding: content_encoding.map(Cow::Borrowed),
                content_type: content_type.map(Cow::Borrowed),
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

    pub fn run(assets_dir: &Path) -> anyhow::Result<()> {
        let output_dir = PathBuf::new()
            .join(env::var_os("OUT_DIR")
                .ok_or_else(|| anyhow::anyhow!("OUT_DIR variable is not defined. You should call this code from a Cargo `build.rs` script"))?)
            .join("edge_frame_assets");

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
            println!("cargo:rustc-env=EDGE_FRAME_ASSET_{}_NAME=", index);
            println!(
                "cargo:rustc-env=EDGE_FRAME_ASSET_{}_DATA={}",
                index,
                output_file.display()
            );
        }

        Ok(())
    }

    pub fn compress(
        assets_dir: &Path,
        output_dir: &Path,
        track: impl Fn(&Path),
    ) -> anyhow::Result<Vec<PathBuf>> {
        let output_files = fs::read_dir(assets_dir)?
            .filter_map(|file| file.ok())
            .filter(|file| file.metadata().map(|md| md.is_file()).unwrap_or(false))
            .map(|file| {
                track(&file.path());

                let output_file =
                    output_dir.join(format!("{}.gz", file.file_name().to_str().unwrap()));

                track(&output_file);

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
