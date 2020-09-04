//!
//! # Get file from ENV
//!
//! ```shell
//! pp env get ...
//! ```
//!

mod pprexec;
mod scp;

use super::{
    super::EnvIdRef,
    run::{get_conn_info, print_to_user, VmConnInfo},
};
use myutil::{err::*, *};
use std::{sync::mpsc::Receiver, time};

///////////////////////////////
#[derive(Default)]
pub struct EnvGet<'a> {
    pub use_ssh: bool,
    pub file_path: &'a str,
    pub env_set: Vec<&'a EnvIdRef>,
    pub time_out: u64,
}
///////////////////////////////

impl<'a> EnvGet<'a> {
    /// 发送请求并打印结果
    pub fn do_req(&self) -> Result<()> {
        self.get_res().c(d!()).and_then(|(n, r)| {
            for _ in 0..n {
                r.recv_timeout(time::Duration::from_secs(self.time_out))
                    .map(print_to_user)
                    .c(d!())?;
            }
            Ok(())
        })
    }

    /// 发送请求并获取结果
    pub fn get_res(&self) -> Result<(usize, Receiver<VmConnInfo>)> {
        if "" == self.file_path {
            return Err(eg!("Empty file_path!"));
        }

        if self.env_set.is_empty() {
            return Err(eg!("Empty env_set!"));
        }

        get_conn_info(&self.env_set).c(d!()).map(|vci_set| {
            if self.use_ssh {
                (vci_set.len(), scp::exec(self.file_path, vci_set))
            } else {
                (vci_set.len(), pprexec::exec(self.file_path, vci_set))
            }
        })
    }
}
