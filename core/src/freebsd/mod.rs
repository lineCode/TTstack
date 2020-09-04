//!
//! # PP, Private Platform.
//!
//! core 模块实现服务端的核心逻辑.
//!
//! Use Bhyve + IPFW + ZFS on FreeBSD.
//!

#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

pub(crate) mod nat;
pub(crate) mod vm;

#[cfg(not(feature = "testmock"))]
use crate::CLONE_MARK;
use crate::{ImagePath, OsName};
use myutil::{err::*, *};
use std::collections::HashMap;
#[cfg(not(feature = "testmock"))]
use std::{
    fs,
    path::{Path, PathBuf},
};

/////////////////
// Entry Point //
/////////////////

/// 全局入口, 必须首先调用
#[inline(always)]
pub fn exec(cb: fn() -> Result<()>, serv_ip: &str) -> Result<()> {
    EntryPoint::new().exec(cb, serv_ip).c(d!())
}

struct EntryPoint;

impl EntryPoint {
    fn new() -> Self {
        EntryPoint
    }

    fn exec(self, cb: fn() -> Result<()>, serv_ip: &str) -> Result<()> {
        nat::init(serv_ip)
            .c(d!())
            .and_then(|_| vm::init().c(d!()))
            .and_then(|_| cb().c(d!()))
    }
}

impl Drop for EntryPoint {
    fn drop(&mut self) {
        info_omit!(vm::env_clean());
    }
}

//////////////////
// Support List //
//////////////////

/// A mocker for tests.
#[cfg(feature = "testmock")]
pub fn get_os_info(img_path: &str) -> Result<HashMap<OsName, ImagePath>> {
    super::test::get_os_info(img_path).c(d!())
}

/// 获取服务端支持的系统列表和对应的 Vm 镜像路径
#[cfg(not(feature = "testmock"))]
pub fn get_os_info(img_path: &str) -> Result<HashMap<OsName, ImagePath>> {
    get_image_path(img_path).c(d!()).map(|path| {
        path.iter()
            .filter_map(|i| i.file_name())
            .filter_map(|i| i.to_str())
            .filter(|i| !i.contains('@') && !i.starts_with(CLONE_MARK))
            .map(|i| i.to_lowercase())
            .zip(path.iter().map(|i| i.to_string_lossy().into_owned()))
            .collect()
    })
}

// 读取 ImagePath 下的所有 zfs volume
#[inline(always)]
#[cfg(not(feature = "testmock"))]
fn get_image_path(img_path: &str) -> Result<Vec<PathBuf>> {
    let mut res = vct![];
    let dir = Path::new(img_path);
    if dir.is_dir() {
        for entry in fs::read_dir(dir).c(d!())? {
            let entry = entry.c(d!())?;
            let path = entry.path();
            if let Some(p) = path.to_str() {
                res.push(PathBuf::from(p));
            }
        }
    }
    Ok(res)
}
