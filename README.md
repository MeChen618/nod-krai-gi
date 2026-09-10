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
