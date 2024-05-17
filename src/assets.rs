use rust_embed::Embed;


#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/static"]
struct Asset;

pub fn get_static(name: &str) -> Option<String> {
    Asset::get(name)
        .and_then(|a| std::str::from_utf8(a.data.as_ref()).ok().map(|s| s.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_artifact() {
        assert!(get_static("stylesheet.css").is_some());
        assert!(get_static("nonexistent.css").is_none());
    }
}