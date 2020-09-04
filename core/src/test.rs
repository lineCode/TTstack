#![cfg(feature = "testmock")]

use crate::{ImagePath, OsName};
use myutil::{err::*, *};
use std::collections::HashMap;

pub(super) fn get_os_info(
    img_path: &str,
) -> Result<HashMap<OsName, ImagePath>> {
    #[cfg(target_os = "linux")]
    const IMG_SUFFIX: &str = ".qcow2";
    #[cfg(target_os = "freebsd")]
    const IMG_SUFFIX: &str = "";

    Ok(map! {
        "CentOS7.0".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 0, IMG_SUFFIX),
        "CentOS7.1".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 1, IMG_SUFFIX),
        "CentOS7.2".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 2, IMG_SUFFIX),
        "CentOS7.3".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 3, IMG_SUFFIX),
        "CentOS7.4".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 4, IMG_SUFFIX),
        "CentOS7.5".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 5, IMG_SUFFIX),
        "CentOS7.6".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 6, IMG_SUFFIX),
        "CentOS7.7".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 7, IMG_SUFFIX),
        "CentOS7.8".to_lowercase() => format!("{}/CentOS7.{}{}", img_path, 8, IMG_SUFFIX),
    })
}
