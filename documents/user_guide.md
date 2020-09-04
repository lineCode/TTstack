# pp client

> **仅支持 Linux 与 MacOS**

## 编译

```shell
make install
```

## 使用

用法:

```shell
pp 0.1.0
范辉 <fanhui.x@gmail.com>


USAGE:
    pp [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    config
    env
    help      Prints this message or the help of the given subcommand(s)
    status
```

#### 配置 pp 服务端的地址

用法:

```shell
pp-config

USAGE:
    pp config [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --client-id <NAME>      客户端别名.
    -a, --server-addr <ADDR>    服务端的监听地址.
    -p, --server-port <PORT>    服务端的监听端口.
```

示例:

```shell
pp config --server-addr=192.168.3.22 [--server-port=9527]
```

#### 创建环境

用法:

```shell
pp-env-add

USAGE:
    pp env add [ENV] [OPTIONS]
    #OR#
    pp env add [OPTIONS] -- [ENV]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -C, --cpu-num <CPU_SIZE>       虚拟机的 CPU 核心数量.
    -D, --disk-size <DISK_SIZE>    虚拟机的磁盘容量, 单位: MB.
    -d, --dup-each <NUM>           每种虚拟机类型启动的实例数量.
    -l, --life-time <TIME>         虚拟机的生命周期, 单位: 秒.
    -M, --mem-size <MEM_SIZE>      虚拟机的内存容量, 单位: MB.
    -s, --os-prefix <OS>... 目标系统名称的前缀.
    -p, --vm-port <PORT>... 虚拟机需要开放的网络端口.
    -t, --vm-type <TYPE>... 虚拟机的类型, 默认为 Qemu.

ARGS:
    <ENV>    待创建的环境名称.
```

注意:

- 若不指定`--mem-size`, 内存默认为 2 GB
- 若不指定`--cpu-num`, CPU 默认为 4 Core
- 若不指定`--vm-port`, 默认只开放 22 端口
- 若不指定`--dup-each`, 默认为 0, 即每种系统只创建一个实例
    - `--dup-each=1` 指每种系统额外多创建一个实例, 即每种系统创建两个实例
    - 指定为其它数据类同, 都是"增加多少倍"的含义
- 新创建的 ENV 会有连接失败的情况, 因部分系统启动较慢, 等一分钟再试
- 执行非常耗时的命令时, 建议使用`nohup $CMD >/tmp/log &`的形式启动, 而后通过查看日志获得执行结果

示例:

```shell
# 短参数风格
# 'centos7' 指 CentOS 7.x 全系列
pp env add [-m 1024] [-c = 8] -s centos7 [-p 80 -p 443] [-l 3600] [-d 1] ENV_NAME

# 长参数风格,
# 指定的端口会被影射为外部可访问的有效端口
pp env add [--mem-size=1024] [--cpu-num=8] --os-prefix=centos7.3,ubuntu18.04 [--vm-port=80,443] [--dup-each=2] ENV_NAME
```

#### 删除环境

用法:

```shell
pp-env-del

USAGE:
    pp env del [ENV]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <ENV>... 一个或多个环境名称.
```

注意:

- 环境运行期间产生的所有数据将会丢失

示例:

```shell
# 删除指定的一个或多个环境
pp env del ENV_1 ENV_2 ENV_3
```

#### 查看环境属性

用法:

```shell
pp-env-list

显示所有已创建的环境列表

USAGE:
    pp env list

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

用法:

```shell
pp-env-show

查看指定环境的详细信息: 主机列表、生命周期等

USAGE:
    pp env show [ENV]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <ENV>... 一个或多个环境名称.
```

#### 修改环境属性

用法:

```shell
pp-env-update

USAGE:
    pp env update [FLAGS] [OPTIONS] <ENV>...

FLAGS:
        --I-am-a-crazy-monkey-AND-I-am-crazy-now    使用此标志指定任意生命周期.
    -h, --help                                      Prints help information
    -V, --version                                   Prints version information

OPTIONS:
    -k, --kick-vm <OS_PREFIX>... 待剔除的系统名称前缀.
    -l, --life-time <TIME>          新的生命周期.

ARGS:
    <ENV>... 一个或多个环境名称.
```

示例:

```shell
# 生命周期默认 1 小时, 最长 6 个小时
pp env update --life-time=$[6 * 3600] ENV_1 ENV_2

# 排除指定的系统
pp env update --kick-vm=centos7.0,ubuntu12 ENV_1 ENV_2
```

#### 往 VM 中布署文件(如: 推送产品包)

用法:

```shell
pp-env-push

将产品包布署到指定一个或多个环境中, 上传的文件位于 /tmp/ 目录下

USAGE:
    pp env push [FLAGS] [OPTIONS] <ENV>...

FLAGS:
    -h, --help       Prints help information
        --use-ssh    使用 SSH 协议通信.
    -V, --version    Prints version information

OPTIONS:
    -f, --file-path <PATH>    文件在本地的路径.

ARGS:
    <ENV>... 一个或多个环境名称.
```

#### 从 VM 中下载文件

用法:

```shell
pp-env-get

从一个或多个环境中下载指定的文件

USAGE:
    pp env get [FLAGS] [OPTIONS] <ENV>...

FLAGS:
    -h, --help       Prints help information
        --use-ssh    使用 SSH 协议通信.
    -V, --version    Prints version information

OPTIONS:
    -f, --file-path <PATH>    文件在远程的路径.

ARGS:
    <ENV>... 一个或多个环境名称.

```

#### 执行命令

用法:

```shell
pp-env-run

向指定环境下的所有主机批量执行相同的操作

USAGE:
    pp env run [OPTIONS] [ENV]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cmd <CMD>          SHELL 命令.
    -f, --script <PATH>      脚本文件的本地路径.

ARGS:
    <ENV>... 一个或多个环境名称.
```

示例:

```shell
# 执行一些简短的命令,
# 其中不要使用引号, 容易出现解析错误,
# 复杂的逻辑请写在脚本中, 然后使用'--script'去执行
pp env run --cmd=<简短的命令> ENV_1 ENV_2 ENV_3

# 运行脚本程序
pp env run --script=<脚本程序的本地路径> ENV_1 ENV_2
```

#### 查看系统状态

用法:

```shell
pp-status

USAGE:
    pp status [FLAGS]

FLAGS:
    -c, --client     查看客户端状态.
    -h, --help       Prints help information
    -s, --server     查看服务端状态.
    -V, --version    Prints version information
```

示例:

```shell
# 自身占用的资源
pp status [--client]

# 服务端资源概况
pp status --server
```
