use chrono::{Datelike, NaiveDateTime, Timelike};
use eframe::epaint::ColorImage;
use egui_extras::RetainedImage;
use image::{load_from_memory, EncodableLayout};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, time::SystemTime};

use crate::errors::{DScopeError, DScopeResult};

const INFO_FILE_NAME: &str = "info.json";
const PHOTO_FILE_NAME_PREFIX: &str = "PICT";
const PHOTO_FILE_NAME_SUFFIX: &str = ".jpg";
const PREVIEW_WIDTH: u32 = 128;

pub const MOLE_CENTER_DISTANCE_MAX: f32 = 2.0;
pub const MOLE_SIZE_MAX: f32 = 4.0;
pub const PHOTO_PX_PER_MM: f32 = 1250.0;

pub fn photo_file_name(id: usize) -> String {
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

pub fn photo_file_id(name: &str) -> Option<usize> {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MoleMetrics {
    pub center_x: f32,
    pub center_y: f32,
    pub diameter: f32,
}

impl MoleMetrics {
    pub fn size(&self) -> Option<f32> {
        if self.diameter > 0.0 {
            Some(self.diameter)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoInfo {
    pub time: SystemTime,
    pub notes: String,
    pub mole_metrics: MoleMetrics,
}

impl PhotoInfo {
    pub fn new(time: SystemTime) -> Self {
        Self {
            time,
            notes: String::new(),
            mole_metrics: Default::default(),
        }
    }
}

pub struct Photo {
    pub id: usize,
    pub bytes: Vec<u8>,
    pub preview: RetainedImage,
    pub info: PhotoInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoSetInfo {
    pub name: String,
    pub surname: String,
    pub time: SystemTime,
    pub notes: String,
}

impl Default for PhotoSetInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            surname: Default::default(),
            time: std::time::SystemTime::now(),
            notes: Default::default(),
        }
    }
}

pub struct PhotoSet {
    pub path: PathBuf,
    pub photos: Vec<Photo>,
    pub info: PhotoSetInfo,
}

impl PhotoSet {
    pub fn from_path(path: PathBuf) -> DScopeResult<Self> {
        if !path.is_dir() {
            return Err(DScopeError::expected_directory(
                path.to_string_lossy().to_string(),
            ));
        }

        let files = path.read_dir().map_err(|error| {
            DScopeError::cannot_read_file(error, path.clone().to_string_lossy().to_string())
        })?;
        let mut photos = Vec::new();
        for file in files.into_iter() {
            let file = match file {
                Ok(file) => file,
                Err(_) => continue,
            };

            let id = match photo_file_id(&file.file_name().to_string_lossy().to_string()) {
                Some(id) => id,
                None => continue,
            };

            let metadata = file.metadata().map_err(|error| {
                DScopeError::cannot_read_file(error, file.path().to_string_lossy().to_string())
            })?;
            if metadata.is_dir() {
                continue;
            }
            if metadata.is_symlink() {
                let symlink_metadata = std::fs::symlink_metadata(file.path()).map_err(|error| {
                    DScopeError::cannot_read_file(error, file.path().to_string_lossy().to_string())
                })?;
                if !symlink_metadata.is_file() {
                    continue;
                }
            }

            let time = metadata.modified().map_err(|error| {
                DScopeError::cannot_read_file(error, file.path().to_string_lossy().to_string())
            })?;
            let bytes = std::fs::read(file.path()).map_err(|error| {
                DScopeError::cannot_read_file(error, file.path().to_string_lossy().to_string())
            })?;

            let image = load_from_memory(&bytes)
                .map_err(|error| DScopeError::cannot_decode_image(error, photo_file_name(id)))?;
            let nheight = ((image.height() as f32)
                * ((PREVIEW_WIDTH as f32) / (image.width() as f32)))
                as u32;
            let preview_image = image::imageops::resize(
                &image,
                PREVIEW_WIDTH,
                nheight,
                image::imageops::FilterType::Nearest,
            );
            let preview_color_image = ColorImage::from_rgba_unmultiplied(
                [PREVIEW_WIDTH as usize, nheight as usize],
                preview_image.as_bytes(),
            );
            let preview = RetainedImage::from_color_image(photo_file_name(id), preview_color_image);

            photos.push(Photo {
                id,
                bytes,
                preview,
                info: PhotoInfo::new(time),
            })
        }

        if photos.len() == 0 {
            return Err(DScopeError::no_photos_found(
                path.to_string_lossy().to_string(),
            ));
        }

        let mut photo_set = PhotoSet {
            path,
            photos,
            info: Default::default(),
        };

        let mut info_path = photo_set.path.clone();
        info_path.push(INFO_FILE_NAME);
        if info_path.exists() {
            let info_text = std::fs::read_to_string(&info_path).map_err(|error| {
                DScopeError::cannot_read_file(error, info_path.to_string_lossy().to_string())
            })?;
            let info_data = serde_json::from_str(&info_text).map_err(|error| {
                DScopeError::cannot_decode_info(error, info_path.to_string_lossy().to_string())
            })?;
            photo_set.apply_data(info_data);
        }

        Ok(photo_set)
    }

    pub fn save(&self) -> DScopeResult<()> {
        for photo in self.photos.iter() {
            let mut path = self.path.clone();
            path.push(photo_file_name(photo.id));
            if !path.exists() {
                std::fs::write(&path, &photo.bytes).map_err(|error| {
                    DScopeError::cannot_write_file(error, path.to_string_lossy().to_string())
                })?;
            }
        }

        let data = serde_json::to_vec_pretty(&self.build_data()).unwrap();
        let mut data_path = self.path.clone();
        data_path.push(INFO_FILE_NAME);
        std::fs::write(&data_path, data).map_err(|error| {
            DScopeError::cannot_write_file(error, data_path.to_string_lossy().to_string())
        })?;

        Ok(())
    }

    fn apply_data(&mut self, data: PhotoSetData) {
        self.info.name = data.name;
        self.info.surname = data.surname;
        self.info.time = data.time;
        self.info.notes = data.notes;
        for (id, info) in data.photos {
            if let Some(photo) = self.photos.get_mut(id) {
                photo.info.time = info.time;
                photo.info.notes = info.notes;
            }
        }
    }

    fn build_data(&self) -> PhotoSetData {
        PhotoSetData {
            name: self.info.name.clone(),
            surname: self.info.surname.clone(),
            time: self.info.time,
            notes: self.info.notes.clone(),
            photos: self.photos.iter().fold(BTreeMap::new(), |mut map, photo| {
                map.insert(photo.id, photo.info.clone());
                map
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoSetData {
    pub name: String,
    pub surname: String,
    pub time: std::time::SystemTime,
    pub notes: String,
    pub photos: BTreeMap<usize, PhotoInfo>,
}

pub struct DisplayTime(SystemTime);

impl DisplayTime {
    pub fn new(time: SystemTime) -> Self {
        Self(time)
    }
}

impl std::fmt::Display for DisplayTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elapsed = self.0.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let time = NaiveDateTime::from_timestamp(elapsed.as_secs() as i64, 0);
        f.write_fmt(format_args!(
            "{:4}-{:2}-{:2} {:2}:{:2}",
            time.year(),
            time.month(),
            time.day(),
            time.time().hour(),
            time.time().minute(),
        ))
    }
}
