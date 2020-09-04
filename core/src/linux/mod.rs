//!
//! # PP, Private Platform.
//!
//! core 模块实现服务端的核心逻辑.
//!
//! use Qemu/KVM + Nftables on Linux.
//!

#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

pub(crate) mod nat;
pub(crate) mod vm;

#[cfg(not(feature = "testmock"))]
#[cfg(feature = "zfs_snapshot")]
use crate::CLONE_MARK;
use crate::{ImagePath, OsName};
use myutil::{err::*, *};
use nix::sched::{clone, CloneFlags};
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
pub fn exec(cb: fn() -> Result<()>, serv_ip: &str) -> Result<()> {
    const STACK_SIZ: usize = 1024 * 1024;
    let mut stack = Vec::with_capacity(STACK_SIZ);
    unsafe {
        stack.set_len(STACK_SIZ);
    }

    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWPID);

    let ops = || -> isize {
        info!(
            vm::util::register_subreaper()
                .c(d!())
                .and_then(|_| vm::util::mount_make_rprivate().c(d!()))
                .and_then(|_| vm::util::mount_dynfs_proc().c(d!()))
                .and_then(|_| vm::util::mount_tmp_tmpfs().c(d!()))
                .and_then(|_| vm::engine::init().c(d!()))
                .and_then(|_| vm::cgroup::init().c(d!()))
                .and_then(|_| nat::init(serv_ip).c(d!()))
                .and_then(|_| cb().c(d!()))
        )
        .and(Ok(0))
        .or::<Result<i32>>(Ok(-1))
        .unwrap()
    };

    clone(
        Box::new(ops),
        stack.as_mut_slice(),
        flags,
        Some(libc::SIGCHLD),
    )
    .c(d!())
    .map(|_| ())
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
#[cfg(feature = "zfs_snapshot")]
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

/// 获取服务端支持的系统列表和对应的 Vm 镜像路径
#[cfg(feature = "backing_file")]
#[cfg(not(feature = "testmock"))]
pub fn get_os_info(img_path: &str) -> Result<HashMap<OsName, ImagePath>> {
    get_image_path(img_path).c(d!()).map(|path| {
        path.iter()
            .filter_map(|i| i.file_name())
            .filter_map(|i| i.to_str())
            .filter(|i| i.ends_with(".qcow2"))
            .map(|i| i.trim_end_matches(".qcow2").to_lowercase())
            .zip(path.iter().map(|i| i.to_string_lossy().into_owned()))
            .collect()
    })
}

/// 读取 zfs snapshot 集合
#[cfg(feature = "zfs_snapshot")]
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

/// 递归读取 ImagePath 下的所有以 ".qcow2" 结尾的文件
#[inline(always)]
#[cfg(feature = "backing_file")]
#[cfg(not(feature = "testmock"))]
fn get_image_path(img_path: &str) -> Result<Vec<PathBuf>> {
    walk_gen(&Path::new(img_path)).c(d!())
}

// recursive function
#[inline(always)]
#[cfg(feature = "backing_file")]
#[cfg(not(feature = "testmock"))]
fn walk_gen(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut res = vct![];

    if dir.is_dir() {
        for entry in fs::read_dir(dir).c(d!())? {
            let entry = entry.c(d!())?;
            let path = entry.path();
            if path.is_dir() {
                res.append(&mut walk_gen(&path).c(d!())?);
            } else if let Some(p) = path.to_str() {
                if p.ends_with(".qcow2") {
                    res.push(PathBuf::from(p));
                }
            }
        }
    }

    Ok(res)
}
