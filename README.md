# nod-krai-gi

[English Version](README.en.md) | [中文版本](README.md)

[手册](handbook.md)

## 简介

`nod-krai-gi` 分叉自 [mavuika-rs](https://git.xeondev.com/mavuika-rs/mavuika-rs) 项目。仅实验性实现。

 base game version : 6.6.0,使用test_gen_handbook 生成手册

## 功能实现/差异
- `sdk-server`: 从项目中删除，使用hoyo-sdk(添加dispatch的重定向，因为从补丁中移除了http，只能使用代理),设置 http_addr = "127.0.0.1:21000"
- `nod-krai-gi-database`: 使用 rocksdb
- `nod-krai-gi-bannber`: 卡池...
- `nod-krai-gi-proto`: 修改为动态加载协议文件
- `nod-krai-gi-ability`: 实现能力系统（错误实现）
- `nod-krai-gi-muip-server`: 提供指令，用于调试。（未完成，编译时删除此部分）

## 构建

需要 stable 工具链，以及以下系统依赖：

```
protobuf-compiler   # nod-krai-gi-proto 的 build script 需要 protoc
clang / libclang    # rocksdb 绑定
cmake, cc           # rocksdb / mlua(vendored)
```

Debian/Ubuntu：`apt-get install -y protobuf-compiler clang libclang-dev cmake build-essential`

之后 `cargo build --workspace` 即可。想用 cranelift 加速 debug 构建需要 nightly，
具体见 `Cargo.toml` 顶部注释。

### 编译 Android 版（Termux 原生运行）

上游 release 的二进制是按 Android 目标编的：

```
$ file nod-krai-gi-game-server
ELF 64-bit LSB pie executable, ARM aarch64, ...,
  interpreter /system/bin/linker64, stripped
```

`/system/bin/linker64` 是 Android 的 Bionic 链接器，因此这些二进制在 Termux 里
原生运行，不需要 proot 或任何 glibc 环境。目标三元组是 `aarch64-linux-android`。

对照：本节下面那套 `aarch64-unknown-linux-gnu` 产物的解释器是
`/lib/ld-linux-aarch64.so.1`，Termux 没有这个文件，直接跑会报
`sh: 1: ./nod-krai-gi-game-server: not found`——这里的 "not found" 指的是
找不到解释器，不是找不到二进制。

有两条路：

**一、在手机上用 Termux 直接编（不需要 NDK）**

Termux 自带完整工具链，本身就是 Android 环境，编出来就是原生的：

```bash
pkg install rust clang cmake protobuf binutils
cargo build --workspace --release
```

省掉了交叉编译的全部配置。代价是手机上编 289 个 crate 比较慢，
rocksdb 和 mlua 尤其吃时间。

**二、在 PC 上用 NDK 交叉编译**

```bash
# NDK 从 developer.android.com 下载，解压后设 ANDROID_NDK_HOME
rustup target add aarch64-linux-android
cargo install cargo-ndk
cargo ndk -t arm64-v8a build --workspace --release
```

`cargo-ndk` 会自动把 NDK 的 clang 配置给 rocksdb 与 mlua 的 build script。
不用它的话需要手动设 `CC_aarch64_linux_android` / `CXX_aarch64_linux_android`
等一系列变量，并让 bindgen 指向 NDK 的 sysroot。

> 以上两种方式均未在本仓库的 CI 或开发环境中实际验证过——本环境无法获取 NDK。
> 已验证的只有上游产物确为 `aarch64-linux-android` 目标这一点（见上面的 file 输出）。

### 交叉编译到 ARM64（glibc，需要 proot 或普通 Linux 发行版）

rocksdb（C++）和 mlua（vendored Lua，C）需要跟着一起交叉，所以除了 Rust target
还得装对应的 C/C++ 交叉工具链：

```
apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

`.cargo/config.toml`（该文件被 `.gitignore` 的 `.*/` 规则排除，需自行创建）：

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

然后：

```bash
CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar \
BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_gnu="--sysroot=/usr/aarch64-linux-gnu" \
cargo build --workspace --release --target aarch64-unknown-linux-gnu
```

`BINDGEN_EXTRA_CLANG_ARGS_*` 是给 rocksdb 的 bindgen 指路的，少了它会找不到目标
架构的系统头文件。

产物在 `target/aarch64-unknown-linux-gnu/release/`，用 `file` 确认是
`ELF 64-bit LSB pie executable, ARM aarch64` 而不是 x86-64。

没有 ARM 机器时可以用 qemu 先验一遍：

```bash
apt-get install -y qemu-user-static
qemu-aarch64-static -L /usr/aarch64-linux-gnu \
  target/aarch64-unknown-linux-gnu/release/nod-krai-gi-dispatch-server
```

注意交叉工具链决定了产物的 glibc 下限（Ubuntu 24.04 的工具链为 2.39），目标机器
的 glibc 低于这个值会报 `version GLIBC_x.yz not found`。

## 资源要求

### 协议文件

服务端启动时扫描 `assets/proto/` 下的每个子目录，每个目录代表一个客户端版本，
且**必须同时具备四个文件**，缺任意一个该目录会被整个跳过：

```
assets/proto/VERSION/
├── all.proto            协议定义
├── protocol.json        cmd_id <-> 消息名
├── version_config.json  ty_value，用于 entity_id >> ty_value 取实体类型
└── replace_value.json   部分字段的混淆参数
```

版本识别不看目录名，而是拿首包的 cmd_id 反查哪个版本的 `protocol.json` 把它映射到
`GetPlayerTokenReq`，因此 `protocol.json` 必须准确。

原 [hk4e-protos](https://gitlab.com/kitkat-multiverse/genshin-protocol) 仓库已失效（404），
可用的镜像有 [rafs-kk/genshin-protocol](https://gitlab.com/rafs-kk/genshin-protocol)
与 [CarolBicsi/genshin-protocol](https://gitlab.com/CarolBicsi/genshin-protocol)，
但目前都只更新到 6.6.0。

拿到一份带 `// CmdId: N` 注释的 dump 后，可以用仓库自带的工具生成上面四个文件：

```
python3 tools/gen_proto_assets.py <dump.proto> <版本号> --fix-duplicates
```

`--fix-duplicates` 用于消除 dump 里的重名定义（dump 常把嵌套 enum 拍平到顶层，
直接使用会解析失败）。生成后建议用 `protoc --proto_path=<目录> --descriptor_set_out=/dev/null <目录>/all.proto`
验证一次。

注意 `replace_value.json` 只有在 dump 带 `(ys_custom).value_mask` 注解时才能自动生成；
这些常量每个版本都会重洗，且服务端在进场景链路上（`enter_scene_token`、`scene_id`、
`retcode` 等）会用到，值不对会进不去游戏。

### 游戏数据

从 [AnimeGameData](https://gitlab.com/Dimbreath/AnimeGameData) 仓库获取游戏数据，并放置在以下位置：

```
assets/BinOutput/
assets/ExcelBinOutput/
```


## 致谢

- 感谢 [mavuika-rs](https://git.xeondev.com/mavuika-rs/mavuika-rs) 项目提供的基础代码
- 感谢 [hk4e-protos](https://gitlab.com/kitkat-multiverse/genshin-protocol) 项目提供的协议文件
- 感谢 [AnimeGameData](https://gitlab.com/Dimbreath/AnimeGameData) 项目提供的游戏数据
