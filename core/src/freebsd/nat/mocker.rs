//!
//! # NAT mocker
//!

use crate::{Ipv4, PubPort, VmPort};
use myutil::err::*;
use std::collections::HashMap;

pub(in crate::freebsd) fn init(_serv_ip: &str) -> Result<()> {
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
