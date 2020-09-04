//!
//! # Config Request
//!

use super::*;
use crate::{read_cfg, write_cfg, Server};
use myutil::{err::*, *};
use std::net::SocketAddr;

///////////////////////////////
#[derive(Default, Debug)]
pub struct Config<'a> {
    pub server_addr: &'a str,
    pub server_port: u16,
    pub client_id: &'a str,
}
///////////////////////////////

impl<'a> Config<'a> {
    /// - 本地信息直接写入配置文件
    /// - 更新 server 信息时, 需验证其有效性
    pub fn do_req(&mut self) -> Result<()> {
        if "" != self.server_addr {
            alt!(0 == self.server_port, self.server_port = 9527);
            get_ops_id("register_client_id")
                .c(d!())
                .and_then(|ops_id| {
                    format!("{}:{}", self.server_addr, self.server_port)
                        .parse::<SocketAddr>()
                        .map(|addr| (ops_id, addr))
                        .c(d!("Invalid server_addr OR server_port"))
                })
                .and_then(|(ops_id, addr)| {
                    send_req::<&str>(ops_id, Req::default(), addr).c(d!())
                })
                .and_then(|_| read_cfg().c(d!()))
                .and_then(|mut cfg| {
                    cfg.server_list =
                        Server::new(self.server_addr, self.server_port);
                    write_cfg(&cfg).c(d!())
                })?;
        }

        if "" != self.client_id {
            read_cfg().c(d!()).and_then(|mut cfg| {
                cfg.client_id = self.client_id.to_owned();
                write_cfg(&cfg).c(d!())
            })?;
        }

        Ok(())
    }
}
