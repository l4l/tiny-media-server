use std::path::{Path, PathBuf};

const SUPPORTED_FORMATS: &[&str] = &["mp4", "gif", "webm", "ogg", "avi", "mkv", "mpeg"];

pub struct MediaPath(PathBuf);

impl std::ops::Deref for MediaPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MediaPath {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self(root.as_ref().to_owned())
    }

    pub fn base_dir(&self) -> &Path {
        self.0.as_path()
    }

    /// Return a list of relative paths of all supported media.
    pub fn media_list(&self) -> Vec<PathBuf> {
        let base = self.base_dir();

        walkdir::WalkDir::new(&base)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| is_supported_format(entry.path()))
            .map(|e| e.path().strip_prefix(&base).unwrap().to_owned())
            .collect::<Vec<_>>()
    }
}

fn is_supported_format(path: &Path) -> bool {
    path.extension()
        .map(|format| {
            SUPPORTED_FORMATS
                .iter()
                .any(|supported_format| format.eq_ignore_ascii_case(supported_format))
        })
        .unwrap_or_default()
}
