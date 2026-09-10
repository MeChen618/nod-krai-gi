use std::collections::HashMap;

// ============================================================================
// 内部辅助结构
// ============================================================================

struct FieldMap {
    map: HashMap<String, String>,
}

impl FieldMap {
    fn take<T: std::str::FromStr>(&mut self, key: &str) -> Result<T, String> {
        let v = self
            .map
            .remove(key)
            .ok_or_else(|| format!("缺少参数: {}", key))?;
        v.parse::<T>()
            .map_err(|_| format!("参数 {} 的值无效: \"{}\"", key, v))
    }

    fn take_opt<T: std::str::FromStr>(&mut self, key: &str) -> Option<T> {
        self.map.remove(key)?.parse().ok()
    }

    fn take_map_u32(&mut self, key: &str) -> Result<HashMap<u32, u32>, String> {
        let raw = match self.map.remove(key) {
            Some(v) => v,
            None => return Ok(HashMap::new()),
        };

        let mut out = HashMap::new();
        for (i, pair) in raw.split(';').enumerate() {
            let mut kv = pair.split(',');
            let k = kv.next().ok_or_else(|| {
                format!("参数 {} 第 {} 组缺少 key", key, i + 1)
            })?;
            let v = kv.next().ok_or_else(|| {
                format!("参数 {} 第 {} 组缺少 value", key, i + 1)
            })?;
            let parsed_k: u32 = k.parse().map_err(|_| {
                format!("参数 {} 第 {} 组的 key 无效: \"{}\"", key, i + 1, k)
            })?;
            let parsed_v: u32 = v.parse().map_err(|_| {
                format!("参数 {} 第 {} 组的 value 无效: \"{}\"", key, i + 1, v)
            })?;
            out.insert(parsed_k, parsed_v);
        }
        Ok(out)
    }

    #[allow(unused)]
    fn take_map_f32(&mut self, key: &str) -> Result<HashMap<u32, f32>, String> {
        let raw = match self.map.remove(key) {
            Some(v) => v,
            None => return Ok(HashMap::new()),
        };

        let mut out = HashMap::new();
        for (i, pair) in raw.split(';').enumerate() {
            let mut kv = pair.split(',');
            let k = kv.next().ok_or_else(|| {
                format!("参数 {} 第 {} 组缺少 key", key, i + 1)
            })?;
            let v = kv.next().ok_or_else(|| {
                format!("参数 {} 第 {} 组缺少 value", key, i + 1)
            })?;
            let parsed_k: u32 = k.parse().map_err(|_| {
                format!("参数 {} 第 {} 组的 key 无效: \"{}\"", key, i + 1, k)
            })?;
            let parsed_v: f32 = v.parse().map_err(|_| {
                format!("参数 {} 第 {} 组的 value 无效: \"{}\"", key, i + 1, v)
            })?;
            out.insert(parsed_k, parsed_v);
        }
        Ok(out)
    }
}

// ============================================================================
// 内部解析辅助函数
// ============================================================================

fn parse_single<'a, I>(mut parts: I, help: &str) -> Result<String, String>
where
    I: Iterator<Item = &'a str>,
{
    parts
        .next()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("参数错误\n用法: {}", help))
}

fn parse_single_u32<'a, I>(mut parts: I, param_name: &str, help: &str) -> Result<u32, String>
where
    I: Iterator<Item = &'a str>,
{
    let raw = parts
        .next()
        .ok_or_else(|| format!("缺少参数: {}\n用法: {}", param_name, help))?;
    raw.parse::<u32>()
        .map_err(|_| format!("参数 {} 的值无效: \"{}\"\n用法: {}", param_name, raw, help))
}

fn parse_struct<'a, I, F, T>(
    mut parts: I,
    first_param: &str,
    help: &str,
    build: F,
) -> Result<T, String>
where
    I: Iterator<Item = &'a str>,
    F: FnOnce(FieldMap) -> Result<T, String>,
{
    let mut map = HashMap::new();

    if let Some(first_val) = parts.next() {
        map.insert(first_param.to_string(), first_val.to_string());
    } else {
        return Err(format!("缺少参数: {}\n用法: {}", first_param, help));
    }

    while let Some(key) = parts.next() {
        let value = parts
            .next()
            .ok_or_else(|| format!("参数 {} 缺少值\n用法: {}", key, help))?;
        map.insert(key.to_string(), value.to_string());
    }

    let fm = FieldMap { map };
    build(fm).map_err(|e| format!("{}\n用法: {}", e, help))
}

// ============================================================================
// 公共类型定义
// ============================================================================

#[allow(unused)]
#[derive(Debug)]
pub enum Command {
    // 角色相关
    Avatar(AvatarAction),
    // buff 操作
    Buff(BuffAction),
    // 物品相关
    Item(ItemAction),
    Weapon(WeaponAction),
    // 任务相关
    Quest(QuestAction),
    // 装置相关
    Gadget(GadgetAction),
    // 群组相关
    Group(GroupAction),
    // 天气与环境
    Weather(u32),
    Climate(u32),
    // 传送相关
    Tp(TpAction),
    // 祈愿相关
    Gacha(GachaAction),
    // 其他
    Prop(String, String),
    SendPacket(String),
    Dun(Option<u32>),
    Pos,
    // help / help <命令>
    Help(Option<String>),
}

// ----------------------------------------------------------------------------
// 角色相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum AvatarAction {
    Add { id: u32 },
    Remove { id: u32 },
    Rename { id: u32, name: String },
    Level { level: u32 },
    Break { break_level: u32 },
    AddTalent { talent_id: u32 },
    Skill { skill_id: u32, level: u32 },
    Elem { element_type: u32 },
    FightProp { key: String, value: f32 },
}

// ----------------------------------------------------------------------------
// buff 操作
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum BuffAction {
    Add { id: u32, level: Option<u32> },
    Clear,
    List,
}

// ----------------------------------------------------------------------------
// 物品相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum ItemAction {
    Add {
        id: u32,
        num: Option<u32>,
        level: Option<u32>,
        refinement: Option<u32>,
        main_prop_id: Option<u32>,
        append_prop_id_list: HashMap<u32, u32>,
    },
    AddMaterial { max_count: Option<u32> },
    AddFurniture { max_count: Option<u32> },
    AddWeapon,
    Clear { target: Option<String> },
    Drop { id: u32 },
}

#[allow(unused)]
#[derive(Debug)]
pub enum WeaponAction {
    Level { level: u32 },
    Break { break_level: u32 },
    Promote { promote_level: u32 },
}

// ----------------------------------------------------------------------------
// 任务相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum QuestAction {
    Accept { id: u32 },
    Finish { id: u32 },
    Cancel { id: u32 },
    Clear { id: Option<u32> },
    State { id: u32, state: u32 },
    Restart { id: u32 },
    RestartAll,
    Var { parent_id: u32, index: Option<u32>, value: Option<u32> },
}

// ----------------------------------------------------------------------------
// 装置相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum GadgetAction {
    Create {
        id: u32,
        num: Option<u32>,
        drop_id: Option<u32>,
        level: Option<u32>,
        interact_id: Option<u32>,
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
    },
    Remove { id: u32 },
    State { id: u32, state: u32 },
    SetStateByEntityId { entity_id: u32, state: u32 },
}

// ----------------------------------------------------------------------------
// 群组相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum GroupAction {
    Refresh { id: u32 },
    Unload { id: u32 },
    Reload { id: u32 },
    Clear { id: u32 },
    SuiteAddExtra { id: u32, suite_id: u32 },
    SuiteRemoveExtra { id: u32, suite_id: u32 },
    SuiteKillExtra { id: u32, suite_id: u32 },
    SuiteGoto { id: u32, suite_id: u32 },
}

// ----------------------------------------------------------------------------
// 传送相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum TpAction {
    A {
        id: u32,
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
    },
    R {
        id: u32,
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
    },
}

// ----------------------------------------------------------------------------
// 祈愿相关
// ----------------------------------------------------------------------------

#[allow(unused)]
#[derive(Debug)]
pub enum GachaAction {
    Add { id: u32 },
    Clear {},
}

// ============================================================================
// 公共解析函数
// ============================================================================

// ============================================================================
// 帮助表
// ============================================================================

/// 一条命令的用法与说明。
///
/// 这张表是 `help` 命令的数据源，与下面 `parse_command` 的分支一一对应——
/// 改解析分支时记得同步这里，`gm_util` 的单元测试会检查两边的命令名集合是否一致。
pub struct CommandHelp {
    /// 一级命令名，如 "avatar"
    pub group: &'static str,
    /// 完整用法，如 "avatar add <avatar_id>"
    pub usage: &'static str,
    /// 一句话说明
    pub desc: &'static str,
}

pub const COMMAND_HELP: &[CommandHelp] = &[
    // 角色
    CommandHelp { group: "avatar", usage: "avatar add <avatar_id>", desc: "添加角色" },
    CommandHelp { group: "avatar", usage: "avatar remove <avatar_id>", desc: "移除角色" },
    CommandHelp { group: "avatar", usage: "avatar rename <avatar_id> <new_name>", desc: "重命名角色" },
    CommandHelp { group: "avatar", usage: "avatar level <level>", desc: "设置当前角色等级" },
    CommandHelp { group: "avatar", usage: "avatar break <break_level>", desc: "设置突破等级" },
    CommandHelp { group: "avatar", usage: "avatar add_talent <talent_id>", desc: "添加命座" },
    CommandHelp { group: "avatar", usage: "avatar skill <skill_id> <level>", desc: "设置技能等级" },
    CommandHelp { group: "avatar", usage: "avatar elem <element_type>", desc: "设置元素类型" },
    CommandHelp { group: "avatar", usage: "avatar fight_prop <key> <value>", desc: "设置战斗属性" },
    // buff
    CommandHelp { group: "buff", usage: "buff add <buff_id> [level <num>]", desc: "添加 buff" },
    CommandHelp { group: "buff", usage: "buff clear", desc: "清空 buff" },
    CommandHelp { group: "buff", usage: "buff list", desc: "列出当前 buff" },
    // 物品
    CommandHelp { group: "item", usage: "item add <item_id> [n <num>] [lv <num>] [r <num>] [m <num>] [p k,v;k,v]", desc: "添加物品；n 数量 lv 等级 r 精炼 m 主词条 p 副词条" },
    CommandHelp { group: "item", usage: "item add material [max_count]", desc: "添加全部材料" },
    CommandHelp { group: "item", usage: "item add furniture [max_count]", desc: "添加全部家具" },
    CommandHelp { group: "item", usage: "item add weapon", desc: "添加全部武器" },
    CommandHelp { group: "item", usage: "item clear [target]", desc: "清空背包，可指定类别" },
    CommandHelp { group: "item", usage: "item drop <item_id>", desc: "在脚下掉落物品" },
    // 武器
    CommandHelp { group: "weapon", usage: "weapon level <level>", desc: "设置武器等级" },
    CommandHelp { group: "weapon", usage: "weapon break <break_level>", desc: "设置武器突破" },
    CommandHelp { group: "weapon", usage: "weapon promote <promote_level>", desc: "设置武器精炼" },
    // 任务
    CommandHelp { group: "quest", usage: "quest accept <quest_id>", desc: "接受任务" },
    CommandHelp { group: "quest", usage: "quest cancel <quest_id>", desc: "取消任务" },
    CommandHelp { group: "quest", usage: "quest finish <quest_id>", desc: "完成任务" },
    CommandHelp { group: "quest", usage: "quest restart <quest_id>", desc: "重置任务" },
    CommandHelp { group: "quest", usage: "quest restart_all", desc: "重置全部任务" },
    CommandHelp { group: "quest", usage: "quest clear", desc: "清空任务" },
    CommandHelp { group: "quest", usage: "quest state <quest_id> <state>", desc: "设置任务状态" },
    CommandHelp { group: "quest", usage: "quest var <parent_id> [index] [value]", desc: "查看/设置任务变量" },
    // 装置
    CommandHelp { group: "gadget", usage: "gadget create <gadget_id> [num <n>] [drop_id <id>] [level <n>] [interact_id <id>] [x <n>] [y <n>] [z <n>]", desc: "生成装置" },
    CommandHelp { group: "gadget", usage: "gadget remove <gadget_id>", desc: "移除装置" },
    CommandHelp { group: "gadget", usage: "gadget state <gadget_id> <state>", desc: "按 gadget_id 设置状态" },
    CommandHelp { group: "gadget", usage: "gadget set_state_by_entity_id <entity_id> <state>", desc: "按 entity_id 设置状态" },
    // 群组
    CommandHelp { group: "group", usage: "group refresh <group_id>", desc: "刷新群组" },
    CommandHelp { group: "group", usage: "group reload <group_id>", desc: "重载群组" },
    CommandHelp { group: "group", usage: "group unload <group_id>", desc: "卸载群组" },
    CommandHelp { group: "group", usage: "group clear <group_id>", desc: "清空群组" },
    CommandHelp { group: "group_suite", usage: "group_suite goto <group_id> <suite_id>", desc: "切换到指定 suite" },
    CommandHelp { group: "group_suite", usage: "group_suite add_extra <group_id> <suite_id>", desc: "追加 suite" },
    CommandHelp { group: "group_suite", usage: "group_suite remove_extra <group_id> <suite_id>", desc: "移除追加的 suite" },
    CommandHelp { group: "group_suite", usage: "group_suite kill_extra <group_id> <suite_id>", desc: "清掉追加 suite 的实体" },
    // 传送
    CommandHelp { group: "tp", usage: "tp a <scene_id> [x <num>] [y <num>] [z <num>]", desc: "传送到绝对坐标" },
    CommandHelp { group: "tp", usage: "tp r <scene_id> [x <num>] [y <num>] [z <num>]", desc: "相对当前位置传送" },
    // 祈愿
    CommandHelp { group: "gacha", usage: "gacha add <gacha_id>", desc: "添加卡池" },
    CommandHelp { group: "gacha", usage: "gacha clear", desc: "清空卡池" },
    // 其他
    CommandHelp { group: "weather", usage: "weather <weather_id>", desc: "设置天气" },
    CommandHelp { group: "climate", usage: "climate <climate_type>", desc: "设置气候" },
    CommandHelp { group: "dun", usage: "dun [id]", desc: "进入副本，省略 id 则退出" },
    CommandHelp { group: "pos", usage: "pos", desc: "显示当前坐标与场景 id" },
    CommandHelp { group: "prop", usage: "prop <key> <value>", desc: "设置玩家属性" },
    CommandHelp { group: "send_packet", usage: "send_packet <key>", desc: "发送调试包" },
    CommandHelp { group: "help", usage: "help [命令]", desc: "列出命令，或查看某条命令的用法" },
];

/// `help` 无参时的输出：按一级命令名列出所有命令组。
pub fn render_help_index() -> String {
    let mut groups: Vec<&str> = Vec::new();
    for h in COMMAND_HELP {
        if !groups.contains(&h.group) {
            groups.push(h.group);
        }
    }
    format!(
        "可用命令（共 {} 条）：\n{}\n\n用 help <命令> 查看具体用法，例如 help avatar",
        COMMAND_HELP.len(),
        groups.join("  ")
    )
}

/// `help <命令>` 的输出。命令名不存在时给出相近提示。
pub fn render_help_topic(topic: &str) -> String {
    let topic = topic.trim().trim_start_matches('/');
    let matched: Vec<&CommandHelp> = COMMAND_HELP.iter().filter(|h| h.group == topic).collect();

    if matched.is_empty() {
        // 前缀相近的候选，便于打错时定位
        let mut near: Vec<&str> = Vec::new();
        for h in COMMAND_HELP {
            if (h.group.starts_with(topic) || topic.starts_with(h.group)) && !near.contains(&h.group) {
                near.push(h.group);
            }
        }
        return if near.is_empty() {
            format!("没有名为 \"{}\" 的命令，用 help 查看全部", topic)
        } else {
            format!("没有名为 \"{}\" 的命令，你是否想找：{}", topic, near.join("  "))
        };
    }

    let mut out = String::new();
    for h in matched {
        out.push_str(&format!("{}\n    {}\n", h.usage, h.desc));
    }
    out.trim_end().to_string()
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    let input = input.trim();
    let input = input.strip_prefix('/').unwrap_or(input);
    let mut parts = input.split_whitespace().peekable();

    let first = parts.next().ok_or("命令不能为空")?;

    // ------------------------------------------------------------------------
    // 单级命令处理
    // ------------------------------------------------------------------------
    match first {
        "prop" => {
            let k = parse_single(&mut parts, "prop <key> <value>")?;
            let v = parse_single(&mut parts, "prop <key> <value>")?;
            return Ok(Command::Prop(k, v));
        }

        "send_packet" => {
            let k = parse_single(&mut parts, "send_packet <key>")?;
            return Ok(Command::SendPacket(k));
        }

        "dun" => {
            let id = parts
                .next()
                .map(|v| {
                    v.parse::<u32>()
                        .map_err(|_| format!("副本 ID 无效: \"{}\"\n用法: dun [id]", v))
                })
                .transpose()?;
            return Ok(Command::Dun(id));
        }

        "pos" => {
            return Ok(Command::Pos);
        }

        "help" => {
            return Ok(Command::Help(parts.next().map(|v| v.to_string())));
        }

        "weather" => {
            let id = parse_single_u32(&mut parts, "weather_id", "weather <weather_id>")?;
            return Ok(Command::Weather(id));
        }

        "climate" => {
            let climate_type =
                parse_single_u32(&mut parts, "climate_type", "climate <climate_type>")?;
            return Ok(Command::Climate(climate_type));
        }

        _ => {}
    }

    let second = parts.next().ok_or_else(|| format!("unknown command: {}", first))?;

    // ------------------------------------------------------------------------
    // 双级命令处理
    // ------------------------------------------------------------------------
    match (first, second) {
        // --------------------------------------------------------------------
        // 角色相关
        // --------------------------------------------------------------------
        ("avatar", "add") => parse_struct(
            parts,
            "avatar_id",
            "avatar add <avatar_id>",
            |mut map| Ok(Command::Avatar(AvatarAction::Add { id: map.take("avatar_id")? })),
        ),

        ("avatar", "remove") => parse_struct(
            parts,
            "avatar_id",
            "avatar remove <avatar_id>",
            |mut map| Ok(Command::Avatar(AvatarAction::Remove { id: map.take("avatar_id")? })),
        ),

        ("avatar", "rename") => {
            let help = "avatar rename <avatar_id> <new_name>";
            let id = parse_single_u32(&mut parts, "avatar_id", help)?;
            let name = parse_single(&mut parts, help)?;
            Ok(Command::Avatar(AvatarAction::Rename { id, name }))
        }

        ("avatar", "level") => parse_struct(
            parts,
            "level",
            "avatar level <level>",
            |mut map| Ok(Command::Avatar(AvatarAction::Level { level: map.take("level")? })),
        ),

        ("avatar", "break") => parse_struct(
            parts,
            "break_level",
            "avatar break <break_level>",
            |mut map| Ok(Command::Avatar(AvatarAction::Break { break_level: map.take("break_level")? })),
        ),

        ("avatar", "add_talent") => parse_struct(
            parts,
            "talent_id",
            "avatar add_talent <talent_id>",
            |mut map| Ok(Command::Avatar(AvatarAction::AddTalent { talent_id: map.take("talent_id")? })),
        ),

        ("avatar", "skill") => {
            let help = "avatar skill <skill_id> <level>";
            let skill_id = parse_single_u32(&mut parts, "skill_id", help)?;
            let level = parse_single_u32(&mut parts, "level", help)?;
            Ok(Command::Avatar(AvatarAction::Skill { skill_id, level }))
        }

        ("avatar", "elem") => parse_struct(
            parts,
            "element_type",
            "avatar elem <element_type>",
            |mut map| Ok(Command::Avatar(AvatarAction::Elem { element_type: map.take("element_type")? })),
        ),

        ("avatar", "fight_prop") => {
            let help = "avatar fight_prop <key> <value>";
            let key = parse_single(&mut parts, help)?;
            let value = parse_single(&mut parts, help)?.parse::<f32>()
                .map_err(|_| format!("value err: {}", help))?;
            Ok(Command::Avatar(AvatarAction::FightProp { key, value }))
        }

        // --------------------------------------------------------------------
        // buff 操作
        // --------------------------------------------------------------------
        ("buff", "add") => parse_struct(
            parts,
            "buff_id",
            "buff add <buff_id> [level <num>]",
            |mut map| Ok(Command::Buff(BuffAction::Add {
                id: map.take("buff_id")?,
                level: map.take_opt("level"),
            })),
        ),

        ("buff", "clear") => Ok(Command::Buff(BuffAction::Clear)),

        ("buff", "list") => Ok(Command::Buff(BuffAction::List)),

        // --------------------------------------------------------------------
        // 物品相关
        // --------------------------------------------------------------------
        ("item", "add") => {
            let next = parts.peek();
            match next {
                Some(&"material") => {
                    parts.next();
                    let max_count = parts.next().and_then(|v| v.parse::<u32>().ok());
                    Ok(Command::Item(ItemAction::AddMaterial { max_count }))
                }
                Some(&"furniture") => {
                    parts.next();
                    let max_count = parts.next().and_then(|v| v.parse::<u32>().ok());
                    Ok(Command::Item(ItemAction::AddFurniture { max_count }))
                }
                Some(&"weapon") => {
                    parts.next();
                    Ok(Command::Item(ItemAction::AddWeapon))
                }
                _ => parse_struct(
                    parts,
                    "item_id",
                    "item add <item_id> [n <num>] [lv <num>] [r <num>] [m <num>] [p k,v;k,v]",
                    |mut map| {
                        Ok(Command::Item(ItemAction::Add {
                            id: map.take("item_id")?,
                            num: map.take_opt("n"),
                            level: map.take_opt("lv"),
                            refinement: map.take_opt("r"),
                            main_prop_id: map.take_opt("m"),
                            append_prop_id_list: map.take_map_u32("p")?,
                        }))
                    },
                ),
            }
        }

        ("item", "clear") => {
            let target = parts.next().map(|v| v.to_string());
            Ok(Command::Item(ItemAction::Clear { target }))
        }

        ("item", "drop") => parse_struct(
            parts,
            "item_id",
            "item drop <item_id>",
            |mut map| Ok(Command::Item(ItemAction::Drop { id: map.take("item_id")? })),
        ),

        ("weapon", "level") => parse_struct(
            parts,
            "level",
            "weapon level <level>",
            |mut map| Ok(Command::Weapon(WeaponAction::Level { level: map.take("level")? })),
        ),

        ("weapon", "break") => parse_struct(
            parts,
            "break_level",
            "weapon break <break_level>",
            |mut map| Ok(Command::Weapon(WeaponAction::Break { break_level: map.take("break_level")? })),
        ),

        ("weapon", "promote") => parse_struct(
            parts,
            "promote_level",
            "weapon promote <promote_level>",
            |mut map| Ok(Command::Weapon(WeaponAction::Promote { promote_level: map.take("promote_level")? })),
        ),

        // --------------------------------------------------------------------
        // 任务相关
        // --------------------------------------------------------------------
        ("quest", "accept") => parse_struct(
            parts,
            "quest_id",
            "quest accept <quest_id>",
            |mut map| Ok(Command::Quest(QuestAction::Accept { id: map.take("quest_id")? })),
        ),

        ("quest", "finish") => parse_struct(
            parts,
            "quest_id",
            "quest finish <quest_id>",
            |mut map| Ok(Command::Quest(QuestAction::Finish { id: map.take("quest_id")? })),
        ),

        ("quest", "cancel") => parse_struct(
            parts,
            "quest_id",
            "quest cancel <quest_id>",
            |mut map| Ok(Command::Quest(QuestAction::Cancel { id: map.take("quest_id")? })),
        ),

        ("quest", "clear") => {
            let id = parts.next().and_then(|v| v.parse::<u32>().ok());
            Ok(Command::Quest(QuestAction::Clear { id }))
        }

        ("quest", "state") => {
            let help = "quest state <quest_id> <state>";
            let id = parse_single_u32(&mut parts, "quest_id", help)?;
            let state = parse_single_u32(&mut parts, "state", help)?;
            Ok(Command::Quest(QuestAction::State { id, state }))
        }

        ("quest", "restart") => parse_struct(
            parts,
            "quest_id",
            "quest restart <quest_id>",
            |mut map| Ok(Command::Quest(QuestAction::Restart { id: map.take("quest_id")? })),
        ),

        ("quest", "restart_all") => Ok(Command::Quest(QuestAction::RestartAll)),

        ("quest", "var") => {
            let help = "quest var <parent_id> [index] [value]";
            let parent_id = parse_single_u32(&mut parts, "parent_id", help)?;
            let index = parts.next().and_then(|v| v.parse::<u32>().ok());
            let value = parts.next().and_then(|v| v.parse::<u32>().ok());
            Ok(Command::Quest(QuestAction::Var {
                parent_id,
                index,
                value,
            }))
        }

        // --------------------------------------------------------------------
        // 装置相关
        // --------------------------------------------------------------------
        ("gadget", "create") => parse_struct(
            parts,
            "gadget_id",
            "gadget create <gadget_id> [num <n>] [drop_id <id>] [level <n>] [interact_id <id>] [x <n>] [y <n>] [z <n>]",
            |mut map| {
                Ok(Command::Gadget(GadgetAction::Create {
                    id: map.take("gadget_id")?,
                    num: map.take_opt("num"),
                    drop_id: map.take_opt("drop_id"),
                    level: map.take_opt("level"),
                    interact_id: map.take_opt("interact_id"),
                    x: map.take_opt("x"),
                    y: map.take_opt("y"),
                    z: map.take_opt("z"),
                }))
            },
        ),

        ("gadget", "remove") => parse_struct(
            parts,
            "gadget_id",
            "gadget remove <gadget_id>",
            |mut map| Ok(Command::Gadget(GadgetAction::Remove { id: map.take("gadget_id")? })),
        ),

        ("gadget", "state") => {
            let help = "gadget state <gadget_id> <state>";
            let id = parse_single_u32(&mut parts, "gadget_id", help)?;
            let state = parse_single_u32(&mut parts, "state", help)?;
            Ok(Command::Gadget(GadgetAction::State { id, state }))
        }

        ("gadget", "set_state_by_entity_id") => {
            let help = "gadget set_state_by_entity_id <entity_id> <state>";
            let entity_id = parse_single_u32(&mut parts, "entity_id", help)?;
            let state = parse_single_u32(&mut parts, "state", help)?;
            Ok(Command::Gadget(GadgetAction::SetStateByEntityId { entity_id, state }))
        }

        // --------------------------------------------------------------------
        // 群组相关
        // --------------------------------------------------------------------
        ("group", "refresh") => parse_struct(
            parts,
            "group_id",
            "group refresh <group_id>",
            |mut map| Ok(Command::Group(GroupAction::Refresh { id: map.take("group_id")? })),
        ),

        ("group", "unload") => parse_struct(
            parts,
            "group_id",
            "group unload <group_id>",
            |mut map| Ok(Command::Group(GroupAction::Unload { id: map.take("group_id")? })),
        ),

        ("group", "reload") => parse_struct(
            parts,
            "group_id",
            "group reload <group_id>",
            |mut map| Ok(Command::Group(GroupAction::Reload { id: map.take("group_id")? })),
        ),

        ("group", "clear") => parse_struct(
            parts,
            "group_id",
            "group clear <group_id>",
            |mut map| Ok(Command::Group(GroupAction::Clear { id: map.take("group_id")? })),
        ),

        ("group_suite", "add_extra") => {
            let help = "group_suite add_extra <group_id> <suite_id>";
            let id = parse_single_u32(&mut parts, "group_id", help)?;
            let suite_id = parse_single_u32(&mut parts, "suite_id", help)?;
            Ok(Command::Group(GroupAction::SuiteAddExtra { id, suite_id }))
        }

        ("group_suite", "remove_extra") => {
            let help = "group_suite remove_extra <group_id> <suite_id>";
            let id = parse_single_u32(&mut parts, "group_id", help)?;
            let suite_id = parse_single_u32(&mut parts, "suite_id", help)?;
            Ok(Command::Group(GroupAction::SuiteRemoveExtra { id, suite_id }))
        }

        ("group_suite", "kill_extra") => {
            let help = "group_suite kill_extra <group_id> <suite_id>";
            let id = parse_single_u32(&mut parts, "group_id", help)?;
            let suite_id = parse_single_u32(&mut parts, "suite_id", help)?;
            Ok(Command::Group(GroupAction::SuiteKillExtra { id, suite_id }))
        }

        ("group_suite", "goto") => {
            let help = "group_suite goto <group_id> <suite_id>";
            let id = parse_single_u32(&mut parts, "group_id", help)?;
            let suite_id = parse_single_u32(&mut parts, "suite_id", help)?;
            Ok(Command::Group(GroupAction::SuiteGoto { id, suite_id }))
        }

        // --------------------------------------------------------------------
        // 传送相关
        // --------------------------------------------------------------------
        ("tp", "a") => parse_struct(
            parts,
            "id",
            "tp a <id> [x <num>] [y <num>] [z <num>]",
            |mut map| {
                Ok(Command::Tp(TpAction::A {
                    id: map.take("id")?,
                    x: map.take_opt("x"),
                    y: map.take_opt("y"),
                    z: map.take_opt("z"),
                }))
            },
        ),

        ("tp", "r") => parse_struct(
            parts,
            "id",
            "tp r <id> [x <num>] [y <num>] [z <num>]",
            |mut map| {
                Ok(Command::Tp(TpAction::R {
                    id: map.take("id")?,
                    x: map.take_opt("x"),
                    y: map.take_opt("y"),
                    z: map.take_opt("z"),
                }))
            },
        ),

        // --------------------------------------------------------------------
        // 祈愿相关
        // --------------------------------------------------------------------
        ("gacha", "add") => parse_struct(
            parts,
            "gacha_id",
            "gacha add <gacha_id>",
            |mut map| Ok(Command::Gacha(GachaAction::Add { id: map.take("gacha_id")? })),
        ),

        ("gacha", "clear") => Ok(Command::Gacha(GachaAction::Clear {})),

        // --------------------------------------------------------------------
        // 未知命令
        // --------------------------------------------------------------------
        _ => Err(format!("unknown command: {} {}", first, second)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 从 usage 里取出命令路径，即前面不带 <> / [] 的那几个词。
    /// 例如 "avatar add <avatar_id>" -> "avatar add"
    fn command_path(usage: &str) -> String {
        usage
            .split_whitespace()
            .take_while(|w| !w.starts_with('<') && !w.starts_with('['))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// COMMAND_HELP 里的每一条都必须真的能被 parse_command 认出来。
    ///
    /// 参数缺失导致的报错是允许的（这里只喂命令名，不喂参数），
    /// 但不允许出现 "unknown command"——那说明帮助表写了个不存在的命令，
    /// 或者解析分支被改名/删掉了而帮助表没跟上。
    #[test]
    fn help_entries_are_all_parseable() {
        let mut bad = Vec::new();
        for h in COMMAND_HELP {
            let path = command_path(h.usage);
            if let Err(e) = parse_command(&path) {
                if e.starts_with("unknown command") {
                    bad.push(format!("{}  ->  {}", h.usage, e));
                }
            }
        }
        assert!(bad.is_empty(), "帮助表里有解析不了的命令:\n{}", bad.join("\n"));
    }

    /// group 字段必须是 usage 的第一个词，否则 help <命令> 查不到。
    #[test]
    fn help_group_matches_usage_prefix() {
        for h in COMMAND_HELP {
            let first = h.usage.split_whitespace().next().unwrap_or("");
            assert_eq!(h.group, first, "group 与 usage 首词不一致: {:?}", h.usage);
        }
    }

    #[test]
    fn help_index_lists_every_group() {
        let out = render_help_index();
        for h in COMMAND_HELP {
            assert!(out.contains(h.group), "help 索引里缺少 {}", h.group);
        }
    }

    #[test]
    fn help_topic_lists_all_entries_of_that_group() {
        let out = render_help_topic("avatar");
        let n = COMMAND_HELP.iter().filter(|h| h.group == "avatar").count();
        assert_eq!(out.lines().filter(|l| l.starts_with("avatar ")).count(), n);
    }

    #[test]
    fn help_topic_suggests_on_typo() {
        let out = render_help_topic("ava");
        assert!(out.contains("avatar"), "应提示相近命令，实际: {}", out);
        let out = render_help_topic("zzzz");
        assert!(out.contains("help"), "应提示用 help 查看全部，实际: {}", out);
    }

    #[test]
    fn help_parses_with_and_without_topic() {
        assert!(matches!(parse_command("help"), Ok(Command::Help(None))));
        assert!(matches!(parse_command("/help"), Ok(Command::Help(None))));
        match parse_command("help avatar") {
            Ok(Command::Help(Some(t))) => assert_eq!(t, "avatar"),
            other => panic!("解析结果不对: {:?}", other),
        }
    }
}
