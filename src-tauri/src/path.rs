use std::borrow::Borrow;

use typed_path::{Utf8UnixPath, Utf8UnixPathBuf, Utf8WindowsPath, Utf8WindowsPathBuf};

/// Modified from the `path-clean` crate
pub trait Utf8PathExtClean {
    type Output: Borrow<Self>;

    fn clean(&self) -> Self::Output;
}

impl Utf8PathExtClean for Utf8WindowsPath {
    type Output = Utf8WindowsPathBuf;

    fn clean(&self) -> Self::Output {
        let mut out = Vec::new();

        for component in self.components() {
            use typed_path::Utf8WindowsComponent::*;

            match component {
                CurDir => (),
                ParentDir => match out.last() {
                    Some(CurDir) => unreachable!(),
                    Some(RootDir) => (),
                    Some(Normal(_)) => {
                        out.pop();
                    }
                    Some(Prefix(_) | ParentDir) | None => {
                        out.push(component);
                    }
                },
                _ => out.push(component),
            }
        }

        match out.as_slice() {
            [] => Utf8WindowsPathBuf::from("."),
            xs => xs.iter().collect(),
        }
    }
}

impl Utf8PathExtClean for Utf8UnixPath {
    type Output = Utf8UnixPathBuf;

    fn clean(&self) -> Self::Output {
        let mut out = Vec::new();

        for component in self.components() {
            use typed_path::Utf8UnixComponent::*;

            match component {
                CurDir => (),
                ParentDir => match out.last() {
                    Some(CurDir) => unreachable!(),
                    Some(RootDir) => (),
                    Some(Normal(_)) => {
                        out.pop();
                    }
                    Some(ParentDir) | None => {
                        out.push(component);
                    }
                },
                _ => out.push(component),
            }
        }

        match out.as_slice() {
            [] => Utf8UnixPathBuf::from("."),
            xs => xs.iter().collect(),
        }
    }
}
