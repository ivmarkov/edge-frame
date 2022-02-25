use anyhow::Result;

pub fn with_path_segment(uri: &str, segment: &str) -> Result<String> {
    let mut url = url::Url::parse(uri)?;

    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("url cannot be used as a base"))?;

        segments.push(segment);
    }

    Ok(url.into())
}
