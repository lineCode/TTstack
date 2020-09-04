# PP 系统管理员指南

Guide for system admin.

## 代码测试

```shell
make lint
make test
```

## 环境准备

#### Linux

> 使用 Qemu 做为 VM 引擎, 使用 Nftables 做 NAT 端口转发.

环境配置:

- `modprobe tun vhost_net`

组件安装

- `qemu`
- `nftables`

#### FreeBSD

> 使用 Bhyve 做为 VM 引擎, 使用 IPFW 内核极 NAT 做端口转发.

环境配置:

- `sysrc gateway_enable=YES`
- `sysrc firewall_enable=YES`
- `sysrc firewall_nat_enable=YES`
- `echo "net.link.tap.up_on_open=1" >> /etc/sysctl.conf`
- `kldload ipfw ipfw_nat if_bridge if_tap`

组件安装

- `sysutils/bhyve-firmware`

## ppserver 配置

```shell
USAGE:
    ppserver [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
    -V, --version      Prints version information

OPTIONS:
        --cpu-total <NUM>      可以使用的 CPU 核心总数.
        --disk-total <SIZE>    可以使用的磁盘总量, 单位: MB.
        --image-path <PATH>    镜像存放路径.
        --log-path <PATH>      日志存储路径.
        --mem-total <SIZE>     可以使用的内存总量, 单位: MB.
        --serv-addr <ADDR>     服务监听地址.
        --serv-port <PORT>     服务监听端口.
```

## ppproxy 配置

```shell
USAGE:
    ppproxy [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --proxy-addr <ADDR>       ppproxy 地址, eg: 127.0.0.1:19527.
        --server-set <ADDR>... ppserver 地址, eg: 127.0.0.1:9527,192.168.3.101:9527.
```
