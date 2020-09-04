# NAT

#### 创建 map, 用于 dnat 影射

```shell
nft '
    add map TABLE PORT_TO_PORT { type inet_service: inet_service; };
    add map TABLE PORT_TO_IPV4 { type inet_service: ipv4_addr; };
'
```

#### 基于 map 设定 dnat 规则
```shell
nft 'add rule inet TABLE CHAIN dnat tcp dport map @PORT_TO_IPV4: tcp dport map @PORT_TO_PORT'
```

#### 通过增删 map 实现动态路由
```shell
nft '
    add element TABLE PORT_TO_IPV4 { 8080 : 10.10.10.10 };
    add element TABLE PORT_TO_IPV4 { 9999 : 1.1.1.1 };
    add element TABLE PORT_TO_PORT { 8080 : 80 };
    delete element TABLE PORT_TO_IPV4 { 8080, 9999 };
'
```

# FILTER

**Don't do this, it's the duty of system-admin.**

#### 创建 set, 用于开放 input 链端口

```shell
nft 'add set inet TABLE WHITE_LIST { type inet_service; };'
```

#### 基于 set 设定 filter 规则

```shell
nft 'add rule inet TABLE CHAIN tcp dport set @WHITE_LIST accept'
```

#### 通过增删 set 实现动态白名单

```shell
nft '
    add element TABLE WHITE_LIST { 40000, 40001, 50090 };
    delete element TABLE WHITE_LIST { 49999, 51456 };
'
```
