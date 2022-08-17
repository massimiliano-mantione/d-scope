use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::errors::DScopeResult;

const INFO_FILE_NAME: &str = "info.json";
const PHOTO_FILE_NAME_PREFIX: &str = "PICT";
const PHOTO_FILE_NAME_SUFFIX: &str = ".jpg";

fn photo_file_name(id: usize) -> String {
    format!(
        "{}{:04}{}",
        PHOTO_FILE_NAME_PREFIX, id, PHOTO_FILE_NAME_SUFFIX
    )
}

#[test]
fn test_photo_file_name() {
    assert_eq!(&photo_file_name(0), "PICT0000.jpg");
    assert_eq!(&photo_file_name(1), "PICT0001.jpg");
    assert_eq!(&photo_file_name(7), "PICT0007.jpg");
    assert_eq!(&photo_file_name(42), "PICT0042.jpg");
}

fn photo_file_id(name: &str) -> Option<usize> {
    if name.len() < 12 {
        return None;
    }

    if !name
        .to_ascii_uppercase()
        .starts_with(PHOTO_FILE_NAME_PREFIX)
    {
        return None;
    }

    if !name.to_ascii_lowercase().ends_with(PHOTO_FILE_NAME_SUFFIX) {
        return None;
    }

    let mut id_slice = &name[4..8];
    while id_slice.len() > 0 && id_slice.starts_with('0') {
        id_slice = &id_slice[1..];
    }

    if id_slice.len() == 0 {
        Some(0)
    } else {
        id_slice.parse::<usize>().ok()
    }
}

#[test]
fn test_photo_file_id() {
    assert_eq!(photo_file_id("PICT0000.jpg"), Some(0));
    assert_eq!(photo_file_id("PICT0001.jpg"), Some(1));
    assert_eq!(photo_file_id("PICT0007.jpg"), Some(7));
    assert_eq!(photo_file_id("PICT0042.jpg"), Some(42));
    assert_eq!(photo_file_id("RICT0008.jpg"), None);
    assert_eq!(photo_file_id("PICT0008.jpj"), None);
    assert_eq!(photo_file_id("PICT000.jpg"), None);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoInfo {
    pub time: SystemTime,
    pub notes: String,
}

impl PhotoInfo {
    pub fn new(time: SystemTime) -> Self {
        Self {
            time,
            notes: String::new(),
        }
    }
}

pub struct Photo {
    pub id: usize,
    pub bytes: Vec<u8>,
    pub info: PhotoInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoSetInfo {
    pub name: String,
    pub surname: String,
    pub time: SystemTime,
    pub notes: String,
}

pub struct PhotoSet {
    pub path: PathBuf,
    pub photos: Vec<Photo>,
    pub info: PhotoSetInfo,
}

impl PhotoSet {
    pub fn from_path(path: PathBuf) -> DScopeResult<Self> {
        todo!()
    }

    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    pub fn save(&self) -> DScopeResult<()> {
        todo!()
    }

    fn apply_data(&mut self, data: PhotoSetData) {
        todo!()
    }

    fn build_data(&self) -> PhotoSetData {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoSetData {
    pub name: String,
    pub surname: String,
    pub time: std::time::SystemTime,
    pub notes: String,
    pub photos: BTreeMap<usize, PhotoInfo>,
}
