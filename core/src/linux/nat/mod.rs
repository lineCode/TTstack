//!
//! # NAT
//!
//! 只处理必要的 NAT 逻辑, 不要配置过滤规则, 那是系统管理员的工作.
//!

#[cfg(not(feature = "testmock"))]
pub(crate) use real::*;

#[cfg(feature = "testmock")]
pub(crate) use mocker::*;

#[cfg(not(feature = "testmock"))]
pub(crate) mod real {
    use crate::{Ipv4, PubPort, VmPort};
    use lazy_static::lazy_static;
    use myutil::{err::*, *};
    use parking_lot::Mutex;
    use std::{
        collections::HashMap, mem, process, sync::Arc, thread, time::Duration,
    };

    const TABLE_PROTO: &str = "ip";
    const TABLE_NAME: &str = "pp-core";

    lazy_static! {
        static ref RULE_SET: Arc<Mutex<String>> =
            Arc::new(Mutex::new(String::new()));
    }

    // nftables 初始化
    pub(in crate::linux) fn init(serv_ip: &str) -> Result<()> {
        set_rule_cron();

        let arg = format!("
            add table {proto} {table};
            flush table {proto} {table};

            add map {proto} {table} PORT_TO_PORT {{ type inet_service: inet_service; }};
            add map {proto} {table} PORT_TO_IPV4 {{ type inet_service: ipv4_addr; }};

            add chain {proto} {table} DNAT_CHAIN {{ type nat hook prerouting priority -100; }};
            add chain {proto} {table} SNAT_CHAIN {{ type nat hook postrouting priority 100; }};

            add rule {proto} {table} DNAT_CHAIN dnat tcp dport map @PORT_TO_IPV4: tcp dport map @PORT_TO_PORT;
            add rule {proto} {table} DNAT_CHAIN dnat udp dport map @PORT_TO_IPV4: udp dport map @PORT_TO_PORT;

            add rule {proto} {table} SNAT_CHAIN ip saddr 10.0.0.0/8 ip daddr != 10.0.0.0/8 snat to {pubip};
            ",
            proto=TABLE_PROTO,
            table=TABLE_NAME,
            pubip=serv_ip,
        );

        nft_exec(&arg).c(d!())
    }

    // 添加新的规则集
    pub(crate) fn set_rule(
        port_map: &HashMap<VmPort, PubPort>,
        vm_ip: &Ipv4,
    ) -> Result<()> {
        if port_map.is_empty() {
            return Ok(());
        }

        let mut port_to_ipv4 = vct![];
        let mut port_to_port = vct![];

        port_map.iter().for_each(|(vmport, pubport)| {
            port_to_ipv4.push(format!("{}:{}", pubport, vm_ip.as_str()));
            port_to_port.push(format!("{}:{}", pubport, vmport));
        });

        let arg = format!(
            "
            add element {proto} {table} PORT_TO_IPV4 {{ {ptoip} }};
            add element {proto} {table} PORT_TO_PORT {{ {ptop} }};
            ",
            proto = TABLE_PROTO,
            table = TABLE_NAME,
            ptoip = port_to_ipv4.join(","),
            ptop = port_to_port.join(","),
        );

        RULE_SET.lock().push_str(&arg);

        Ok(())
    }

    // 清理指定端口对应的 NAT 规则
    pub(crate) fn clean_rule(port_set: &[PubPort]) -> Result<()> {
        if port_set.is_empty() {
            return Ok(());
        }

        let arg = format!(
            "
            delete element {proto} {table} PORT_TO_IPV4 {{ {pub_port} }};
            delete element {proto} {table} PORT_TO_PORT {{ {pub_port} }};
            ",
            proto = TABLE_PROTO,
            table = TABLE_NAME,
            pub_port = port_set
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );

        RULE_SET.lock().push_str(&arg);

        Ok(())
    }

    // 执行 nftables 命令
    #[inline(always)]
    fn nft_exec(arg: &str) -> Result<()> {
        let res = process::Command::new("nft").arg(arg).output().c(d!())?;
        if res.status.success() {
            Ok(())
        } else {
            Err(eg!(String::from_utf8_lossy(&res.stderr)))
        }
    }

    // TODO: 更优雅的实现方式
    //
    // nftables 并发设置规则会概率性失败,
    // 以单独的线程定时应用收集到的所有规则
    fn set_rule_cron() {
        thread::spawn(|| {
            loop {
                let arg = mem::take(&mut *RULE_SET.lock());
                if !arg.is_empty() {
                    info_omit!(nft_exec(&arg));
                }
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}

#[cfg(feature = "testmock")]
pub(crate) mod mocker {
    use crate::{Ipv4, PubPort, VmPort};
    use myutil::err::*;
    use std::collections::HashMap;

    pub(in crate::linux) fn init(_serv_ip: &str) -> Result<()> {
        Ok(())
    }

    pub(crate) fn set_rule(
        _port_map: &HashMap<VmPort, PubPort>,
        _vm_ip: &Ipv4,
    ) -> Result<()> {
        Ok(())
    }

    pub(crate) fn clean_rule(_port_set: &[PubPort]) -> Result<()> {
        Ok(())
    }
}
