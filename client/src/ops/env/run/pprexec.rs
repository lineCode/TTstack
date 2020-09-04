//!
//! A fast implementation based on pprexec.
//!

use super::VmConnInfo;
use myutil::{err::*, *};
use pprexec::client::req_exec;
use std::{
    sync::mpsc::{channel, Receiver},
    thread,
};

pub(super) const PORT: u16 = 22000;

pub(super) fn exec(
    remote_cmd: &str,
    vm_conn_info: Vec<VmConnInfo>,
    timeout_secs: u64,
) -> Receiver<VmConnInfo> {
    let (s, r) = channel();
    vm_conn_info.into_iter().for_each(|mut vci| {
        let cmd = remote_cmd.to_owned();
        let sender = s.clone();
        thread::spawn(move || {
            match req_exec(
                &format!("{}:{}", vci.addr, vci.pprexec_port),
                &cmd,
                Some(timeout_secs),
            ) {
                Ok(resp) => {
                    vci.stdout = resp.stdout.into_owned();
                    vci.stderr = resp.stderr.into_owned();
                }
                Err(e) => {
                    vci.stderr = genlog(e);
                }
            }
            info_omit!(sender.send(vci));
        });
    });

    r
}
