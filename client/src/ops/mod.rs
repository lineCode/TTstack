//!
//! 请求类别, 对应服务端的处理函数:
//!
//! ```rust
//! const OPS_MAP: &[Ops] = &[
//!     register_client_id,
//!     get_server_info,
//!     get_env_list,
//!     get_env_info,
//!     add_env,
//!     del_env,
//!     update_env_lifetime,
//!     update_env_kick_vm,
//! ];
//! ```
//!

pub mod config;
pub mod env;
pub mod status;

use lazy_static::lazy_static;
use myutil::{err::*, *};
pub(self) use ppserver_def::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

lazy_static! {
    static ref OPS_MAP: HashMap<&'static str, u8> = map! {
        "register_client_id" => 0,
        "get_server_info" => 1,
        "get_env_list" => 2,
        "get_env_info" => 3,
        "add_env" => 4,
        "del_env" => 5,
        "update_env_lifetime" => 6,
        "update_env_kick_vm" => 7,
    };
    static ref SOCK: UdpSocket = pnk!(gen_sock(8));
}

/// 解析返回的结果
#[macro_export]
macro_rules! resp_parse {
    ($resp_orig: expr, $body_type: ty) => {
        match $resp_orig.status {
            RetStatus::Success => {
                serde_json::from_slice::<$body_type>(&$resp_orig.msg).c(d!())
            }
            RetStatus::Fail => {
                Err(eg!(String::from_utf8_lossy(&$resp_orig.msg)))
            }
        }
    };
}

/// 打印返回的结果
#[macro_export]
macro_rules! resp_print {
    ($body: expr) => {{
        dbg!($body);
    }};
    ($resp_orig: expr, $body_type: ty) => {
        $crate::resp_parse!($resp_orig, $body_type).map(|body| {
            dbg!(body);
        })
    };
}

#[inline(always)]
fn get_ops_id(ops: &str) -> Result<u8> {
    OPS_MAP
        .get(ops)
        .copied()
        .ok_or(eg!(format!("Unknown request: {}", ops)))
}

fn gen_sock(timeout: u64) -> Result<UdpSocket> {
    let mut addr;
    for port in (20_000 + ts!() % 10_000)..60_000 {
        addr = SocketAddr::from(([0, 0, 0, 0], port as u16));
        if let Ok(sock) = UdpSocket::bind(addr) {
            sock.set_read_timeout(Some(Duration::from_secs(timeout)))
                .c(d!())?;
            return Ok(sock);
        }
    }
    Err(eg!())
}

/// 发送请求信息
pub fn send_req<T: Serialize>(
    ops_id: u8,
    req: Req<T>,
    peeraddr: SocketAddr,
) -> Result<Resp> {
    let mut body =
        format!("{id:>0width$}", id = ops_id, width = OPS_ID_LEN).into_bytes();
    body.append(&mut serde_json::to_vec(&req).c(d!())?);

    SOCK.send_to(&body, peeraddr).c(d!()).and_then(|_| {
        let mut buf = vct![0; 8 * 4096];
        let size = SOCK.recv(&mut buf).c(d!())?;
        serde_json::from_slice(&buf[0..size]).c(d!())
    })
}

/// 生成 Req 结构
#[inline(always)]
pub fn gen_req<T: Serialize>(msg: T) -> Req<T> {
    Req::new(0, msg)
}
