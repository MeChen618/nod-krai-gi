#!/usr/bin/env python3
"""把一份 dump 出来的 all.proto 转成 nod-krai-gi 需要的协议目录。

服务端在 crates/nod-krai-gi-proto/src/dy_parser.rs 里扫 assets/proto/<版本>/，
每个目录必须同时具备四个文件，缺一个就整目录跳过（见 update_proto 的 exists 检查）：

    all.proto            协议定义
    protocol.json        cmd_id <-> 消息名，来自 proto 里的 `// CmdId: N` 注释
    version_config.json  ty_value，用于 entity_id >> ty_value 取实体类型
    replace_value.json   部分字段的混淆参数

用法：
    python3 tools/gen_proto_assets.py <dump.proto> <版本号> [--ty-value N]
    python3 tools/gen_proto_assets.py ~/all.proto 6.6.0

关于 replace_value.json：
只有当 dump 里带 `(ys_custom).value_mask = "..."` 注解时才能自动生成。不带注解的
dump 会生成空的 {}，此时服务端对这些字段不做任何变换 —— 如果客户端那一版确实
启用了混淆，进场景会失败。这些常量每个版本都会重洗，必须用对应版本的值。
"""

import argparse
import json
import pathlib
import re
import sys

# `// CmdId: 25580` 或 `// CmdID: 28757`，紧跟着 `message Xxx {`
CMD_ID_RE = re.compile(
    r"^//\s*CmdI[Dd]:\s*(\d+)\s*$\s*^message\s+(\w+)", re.MULTILINE
)

# uint32 team_id = 4 [(ys_custom).is_masked = true, (ys_custom).value_mask = "(val - 65504) ^ 27354"];
MASK_RE = re.compile(
    r"^\s*\w[\w.<>, ]*\s+(\w+)\s*=\s*\d+\s*\[[^\]]*?"
    r'\(ys_custom\)\.value_mask\s*=\s*"([^"]+)"'
)

MESSAGE_OPEN_RE = re.compile(r"^\s*message\s+(\w+)\s*\{")
ENUM_OPEN_RE = re.compile(r"^\s*enum\s+\w+\s*\{")

# dy_parser.rs 里 get_ty_value_by_version 找不到版本时的兜底值
DEFAULT_TY_VALUE = 24

# 服务端实际调用 replace_* 时传入的 key，逐字取自 crates/ 下的调用点。
# 这些 key 的形式是 `消息名.字段名`，所以 replace_value.json 必须按同样的形式建表，
# 否则 dy_parser.rs 的 replace_value_map.get() 一律 miss，混淆等于没生效。
CRITICAL_MASK_KEYS = [
    "EnterSceneDoneRsp.enter_scene_token",
    "EnterSceneDoneRsp.retcode",
    "EnterScenePeerNotify.enter_scene_token",
    "EnterSceneReadyRsp.enter_scene_token",
    "EnterSceneReadyRsp.retcode",
    "PlayerEnterSceneNotify.dungeon_id",
    "PlayerEnterSceneNotify.enter_reason",
    "PlayerEnterSceneNotify.enter_scene_token",
    "PlayerEnterSceneNotify.scene_begin_time",
    "PlayerEnterSceneNotify.scene_id",
    "PlayerEnterSceneNotify.target_uid",
    "PlayerEnterSceneNotify.world_level",
    "PostEnterSceneRsp.enter_scene_token",
    "PostEnterSceneRsp.retcode",
    "SceneInitFinishRsp.enter_scene_token",
    "SceneInitFinishRsp.retcode",
    "SetUpAvatarTeamReq.cur_avatar_guid",
    "SetUpAvatarTeamReq.team_id",
    "SetUpAvatarTeamRsp.cur_avatar_guid",
    "SetUpAvatarTeamRsp.retcode",
    "SetUpAvatarTeamRsp.team_id",
]


# 未被还原的混淆消息名，形如 FHDLHJICDHB
OBFUSCATED_NAME_RE = re.compile(r"^[A-Z]{8,}$")


TOP_LEVEL_BLOCK_RE = re.compile(r"^(message|enum)\s+(\w+)\s*\{")


def split_top_level_blocks(lines):
    """按大括号配平切出顶层的 message / enum 块，返回 (类型, 名字, 起始行, 结束行)。"""
    blocks = []
    i = 0
    while i < len(lines):
        m = TOP_LEVEL_BLOCK_RE.match(lines[i])
        if not m:
            i += 1
            continue
        start, depth = i, 0
        while i < len(lines):
            depth += lines[i].count("{") - lines[i].count("}")
            i += 1
            if depth == 0:
                break
        blocks.append((m.group(1), m.group(2), start, i))
    return blocks


def fix_duplicate_definitions(text):
    """消掉顶层重名，让 protoc / protobuf-parse 能过。

    dump 常把原本嵌套在不同 message 里的 enum 拍平到顶层，于是出现多个同名
    `State` / `Reason`。处理方式分两种：

      * 内容完全一致的，直接删掉后面的副本；
      * 内容不同的，把第二个及之后的重命名成 `名字__dupN`（enum 的值也一起加后缀）。

    重命名只动定义、不动引用，引用仍绑到第一个定义。对 enum 而言这不影响收发：
    proto3 的 enum 在 wire 上就是 varint，编码的是数值本身，与绑定到哪个 enum 类型无关。
    重名的 message 则会真的改变解码结构，所以单独警告出来。
    """
    lines = text.split("\n")
    blocks = split_top_level_blocks(lines)

    by_name = {}
    drop_ranges = []
    renames = []
    for kind, name, start, end in blocks:
        body = "\n".join(lines[start:end])
        if name not in by_name:
            by_name[name] = body
            continue
        if by_name[name] == body:
            drop_ranges.append((start, end))
            continue
        renames.append((kind, name, start, end))

    dup_count = {}
    for kind, name, start, end in renames:
        dup_count[name] = dup_count.get(name, 0) + 1
        new_name = f"{name}__dup{dup_count[name]}"
        if kind == "message":
            print(
                f"  警告: 顶层 message {name} 重复且内容不同，第 {dup_count[name]} 份改名为 "
                f"{new_name}；引用仍指向第一份，如果该消息在收发路径上需人工确认",
                file=sys.stderr,
            )
        lines[start] = lines[start].replace(name, new_name, 1)
        if kind == "enum":
            # enum 的值按 C++ 作用域是全局兄弟，同样要改名
            for i in range(start + 1, end):
                lines[i] = re.sub(
                    r"^(\s*)(\w+)(\s*=\s*-?\d+\s*;)",
                    lambda m: f"{m.group(1)}{m.group(2)}__dup{dup_count[name]}{m.group(3)}",
                    lines[i],
                )

    for start, end in sorted(drop_ranges, reverse=True):
        del lines[start:end]

    if drop_ranges or renames:
        print(
            f"  去重: 删除完全相同的重复定义 {len(drop_ranges)} 处，"
            f"重命名内容不同的重名定义 {len(renames)} 处",
            file=sys.stderr,
        )
    return "\n".join(lines)


# `repeated Uint32Pair affix_list = 3;` / `map<uint32, uint32> group_map = 6;`
FIELD_RE = re.compile(r"^(\s*)((?:repeated|optional|required)\s+)?([\w.]+(?:<[^>]*>)?)(\s+)(\w+)(\s*=\s*\d+\s*;.*)$")
SCOPE_OPEN_RE = re.compile(r"^\s*(message|enum)\s+\w+\s*\{")
# oneof 里的字段属于外层 message 的命名空间，不单独开作用域
NON_SCOPE_OPEN_RE = re.compile(r"^\s*oneof\s+\w+\s*\{")


def fix_duplicate_fields(text):
    """同一 message 内字段重名时给后出现的改名。

    dump 里见过 `MixinRecoverDrawPlayInfo draw_play_info = 9;` 和 oneof 内的
    `DrawPlayInfo draw_play_info = 100;` 撞在同一个 message 下。改名只动名字、
    不动字段号，wire 格式不受影响。
    """
    lines = text.split("\n")
    scopes = [set()]
    renamed = 0

    for idx, line in enumerate(lines):
        if SCOPE_OPEN_RE.match(line):
            scopes.append(set())
            continue
        if NON_SCOPE_OPEN_RE.match(line):
            continue
        if line.strip().startswith("}"):
            if len(scopes) > 1:
                scopes.pop()
            continue

        m = FIELD_RE.match(line)
        if not m:
            continue
        name = m.group(5)
        scope = scopes[-1]
        if name not in scope:
            scope.add(name)
            continue

        new_name = name
        n = 1
        while new_name in scope:
            new_name = f"{name}__dup{n}"
            n += 1
        scope.add(new_name)
        lines[idx] = f"{m.group(1)}{m.group(2) or ''}{m.group(3)}{m.group(4)}{new_name}{m.group(6)}"
        renamed += 1

    if renamed:
        print(f"  去重: message 内重名字段改名 {renamed} 处", file=sys.stderr)
    return "\n".join(lines)


def parse_masks(text):
    """抽取带 (ys_custom).value_mask 的字段，按 `消息名.字段名` 建表。

    裸字段名不行：服务端查的是 `PlayerEnterSceneNotify.enter_scene_token` 这种
    带消息名的 key，而且 enter_scene_token / retcode 这类字段在多个 message 里
    各有各的常量，只用字段名会互相覆盖。
    """
    masks = {}
    stack = []
    for line in text.split("\n"):
        m = MESSAGE_OPEN_RE.match(line)
        if m:
            stack.append(m.group(1))
            continue
        if ENUM_OPEN_RE.match(line):
            stack.append(None)
            continue
        if NON_SCOPE_OPEN_RE.match(line):
            continue
        if line.strip().startswith("}"):
            if stack:
                stack.pop()
            continue

        m = MASK_RE.match(line)
        if not m or not stack:
            continue
        # 取最近的 message 名；嵌套 message 用外层.内层，与 dump 拍平后的写法一致
        owner = next((n for n in reversed(stack) if n), None)
        if owner is None:
            continue
        masks.setdefault(f"{owner}.{m.group(1)}", m.group(2))
    return masks


YS_OPTION_RE = re.compile(r"\(ys_custom\)\.\w+\s*=\s*(?:\"[^\"]*\"|[^,\]]+)\s*,?\s*")


def strip_ys_custom(text):
    """去掉 ys_custom 扩展及其对 descriptor.proto 的 import。

    服务端解析 all.proto 时只把版本目录本身作为 include 路径
    （dy_parser.rs 的 `includes(&[... .parent()])`），解析不到
    google/protobuf/descriptor.proto，会以 DependencyNotFound 崩掉加载线程。

    这套注解的信息已经抽进 replace_value.json，且字段选项属于描述符元数据、
    不参与 wire 编码，删掉不影响收发。
    """
    lines = text.split("\n")
    out, skip_depth, dropped = [], 0, 0

    for line in lines:
        if skip_depth:
            skip_depth += line.count("{") - line.count("}")
            continue
        st = line.strip()
        if st.startswith('import "google/protobuf/descriptor.proto"'):
            dropped += 1
            continue
        if re.match(r"^\s*(message\s+YsCustom|extend\s+google\.protobuf\.FieldOptions)\s*\{", line):
            skip_depth = line.count("{") - line.count("}")
            dropped += 1
            continue
        if "(ys_custom)" in line:
            line = YS_OPTION_RE.sub("", line)
            # 选项全删完后把空的 [] 也去掉
            line = re.sub(r"\[\s*\]", "", line)
            line = re.sub(r"\s+;", ";", line)
        out.append(line)

    if dropped:
        print(f"  已剥离 ys_custom 扩展定义 {dropped} 处", file=sys.stderr)
    return "\n".join(out)


def parse_cmd_ids(text):
    """抽出 消息名 -> cmd_id。

    服务端两个方向都要用（cmd_id_map 与 cmd_id_map_re），所以同一个 cmd_id 落到
    多个消息上时得挑一个。实测 dump 里这种冲突只出现在低位 id 上，且多半有一边是
    没还原出来的混淆名，因此优先保留可读的那个，并把冲突打出来让人自己判断。
    """
    mapping = {}
    owner = {}
    for raw_id, name in CMD_ID_RE.findall(text):
        cmd_id = int(raw_id)
        mapping[name] = cmd_id

        prev = owner.get(cmd_id)
        if prev is None or prev == name:
            owner[cmd_id] = name
            continue

        prev_obf = bool(OBFUSCATED_NAME_RE.match(prev))
        curr_obf = bool(OBFUSCATED_NAME_RE.match(name))
        if prev_obf and not curr_obf:
            owner[cmd_id] = name
        winner = owner[cmd_id]
        print(
            f"  警告: cmd_id {cmd_id} 同时挂在 {prev} 和 {name} 上，反查方向取 {winner}",
            file=sys.stderr,
        )

    # 反查方向只保留胜出的那个，避免 id -> 名字 不确定
    return {name: cid for name, cid in mapping.items() if owner.get(cid) == name}


def parse_mask(expr):
    """把 value_mask 表达式翻成 ReplaceValueConfig 的 n1..n5。

    服务端出站算的是 ((v + n1 - n2) ^ n3) + n4 - n5（dy_parser.rs replace_out_u32），
    入站是它的逆运算。dump 里见过的三种形式都能落进这个式子：

        (val + A) ^ B      -> n1=A, n3=B
        (val - A) ^ B      -> n2=A, n3=B
        (val ^ B) - C      -> n3=B, n5=C
    """
    cfg = {"n1": 0, "n2": 0, "n3": 0, "n4": 0, "n5": 0, "const_in": 0, "const_out": 0}
    e = expr.replace(" ", "")

    m = re.fullmatch(r"\(val([+-])(\d+)\)\^(\d+)", e)
    if m:
        sign, a, b = m.group(1), int(m.group(2)), int(m.group(3))
        cfg["n1" if sign == "+" else "n2"] = a
        cfg["n3"] = b
        return cfg

    m = re.fullmatch(r"\(val\^(\d+)\)([+-])(\d+)", e)
    if m:
        b, sign, c = int(m.group(1)), m.group(2), int(m.group(3))
        cfg["n3"] = b
        cfg["n4" if sign == "+" else "n5"] = c
        return cfg

    m = re.fullmatch(r"val\^(\d+)", e)
    if m:
        cfg["n3"] = int(m.group(1))
        return cfg

    return None


def main():
    ap = argparse.ArgumentParser(description="生成 assets/proto/<版本>/ 四件套")
    ap.add_argument("dump", type=pathlib.Path, help="dump 出来的 .proto")
    ap.add_argument("version", help="版本目录名，例如 6.6.0")
    ap.add_argument("--ty-value", type=int, default=DEFAULT_TY_VALUE)
    ap.add_argument("--out-root", type=pathlib.Path, default=pathlib.Path("assets/proto"))
    ap.add_argument(
        "--fix-duplicates",
        action="store_true",
        help="消除顶层重名定义（dump 把嵌套 enum 拍平后常见），否则解析会失败",
    )
    args = ap.parse_args()

    text = args.dump.read_text(encoding="utf-8", errors="replace")
    if args.fix_duplicates:
        text = fix_duplicate_definitions(text)
        text = fix_duplicate_fields(text)

    cmd_ids = parse_cmd_ids(text)
    if not cmd_ids:
        raise SystemExit("没解析到任何 `// CmdId: N` 注释，这份 dump 不带 cmd_id，无法生成 protocol.json")

    # get_version() 靠首包 cmd_id 反查 GetPlayerTokenReq 来认版本，缺了它这个目录等于废的
    if "GetPlayerTokenReq" not in cmd_ids:
        raise SystemExit("dump 里没有带 cmd_id 的 GetPlayerTokenReq，服务端将无法识别该版本")

    replace_value = {}
    for key, expr in parse_masks(text).items():
        cfg = parse_mask(expr)
        if cfg is None:
            print(f"  警告: 看不懂的 mask 表达式，已跳过 {key} = {expr!r}", file=sys.stderr)
            continue
        replace_value[key] = cfg

    # 必须在 parse_masks 之后：剥离会把注解本身删掉
    text = strip_ys_custom(text)

    out = args.out_root / args.version
    out.mkdir(parents=True, exist_ok=True)
    (out / "all.proto").write_text(text, encoding="utf-8")
    (out / "protocol.json").write_text(
        json.dumps(dict(sorted(cmd_ids.items())), indent=2), encoding="utf-8"
    )
    (out / "version_config.json").write_text(
        json.dumps({"ty_value": args.ty_value}, indent=2), encoding="utf-8"
    )
    (out / "replace_value.json").write_text(
        json.dumps(replace_value, indent=2, sort_keys=True), encoding="utf-8"
    )

    print(f"已写入 {out}")
    print(f"  消息数            {len(cmd_ids)}")
    print(f"  GetPlayerTokenReq {cmd_ids['GetPlayerTokenReq']}")
    print(f"  ty_value          {args.ty_value}")
    print(f"  混淆字段          {len(replace_value)}")

    missing = [k for k in CRITICAL_MASK_KEYS if k not in replace_value]
    if missing:
        print(
            "\n注意: 服务端会对下列字段调用 replace_*，但本次没生成对应参数，"
            "它们将按原值收发：\n  " + ", ".join(missing) +
            "\n如果这一版客户端启用了字段混淆，进场景会失败；需要补上该版本的常量。",
            file=sys.stderr,
        )


if __name__ == "__main__":
    main()
