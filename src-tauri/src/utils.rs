use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use typed_path::{
    Utf8NativePath, Utf8NativePathBuf, Utf8UnixPath, Utf8UnixPathBuf, Utf8WindowsPath,
    Utf8WindowsPathBuf,
};

#[cfg(windows)]
pub fn path_to_bytes(path: &Path) -> Cow<[u8]> {
    use std::os::windows::ffi::OsStrExt;
    let chars = path.as_os_str().encode_wide();
    let bytes = chars.flat_map(u16::to_le_bytes).collect();
    Cow::Owned(bytes)
}

#[cfg(unix)]
pub fn path_to_bytes(path: &Path) -> Cow<[u8]> {
    use std::os::unix::ffi::OsStrExt;
    Cow::Borrowed(path.as_os_str().as_bytes())
}

pub fn get_config_dir() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Failed to get user config dir")?;

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    path.push("Ellisia");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    path.push("ellisia");

    Ok(path)
}

pub fn now_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn clean_path(path: &Utf8NativePath) -> Utf8NativePathBuf {
    #[cfg(windows)]
    return clean_path_windows(path);
    #[cfg(unix)]
    return clean_path_unix(path);
}

pub fn clean_path_windows(path: &Utf8WindowsPath) -> Utf8WindowsPathBuf {
    let mut out = Vec::new();

    for component in path.components() {
        use typed_path::windows::Utf8WindowsComponent::*;
        match component {
            CurDir => (),
            ParentDir => match out.last() {
                Some(RootDir) => (),
                Some(Normal(_)) => {
                    out.pop();
                }
                Some(CurDir | ParentDir | Prefix(_)) | None => {
                    out.push(component);
                }
            },
            component => out.push(component),
        }
    }

    if out.is_empty() {
        Utf8WindowsPathBuf::from(".")
    } else {
        out.iter().collect()
    }
}

pub fn clean_path_unix(path: &Utf8UnixPath) -> Utf8UnixPathBuf {
    let mut out = Vec::new();

    for component in path.components() {
        use typed_path::unix::Utf8UnixComponent::*;
        match component {
            CurDir => (),
            ParentDir => match out.last() {
                Some(RootDir) => (),
                Some(Normal(_)) => {
                    out.pop();
                }
                Some(CurDir | ParentDir) | None => {
                    out.push(component);
                }
            },
            component => out.push(component),
        }
    }

    if out.is_empty() {
        Utf8UnixPathBuf::from(".")
    } else {
        out.iter().collect()
    }
}
