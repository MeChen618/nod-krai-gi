pub use super::talent_types::TalentConfig;
use crate::config::TalentAction;
use crate::excel::{AvatarTalentExcelConfig, ProudSkillExcelConfig};
use common::string_util::InternString;
use std::collections::hash_map::Iter;
use std::{
    collections::HashMap,
    fs::{self, ReadDir},
    sync::OnceLock,
};

static AVATAR_TALENT_CONFIG_MAP: OnceLock<HashMap<InternString, Vec<TalentAction>>> =
    OnceLock::new();

fn load_avatar_talent_configs(talent_config_dir: ReadDir) -> std::io::Result<()> {
    let mut map = HashMap::new();
    for entry in talent_config_dir {
        let entry = entry?;
        let json = match std::fs::read(entry.path()) {
            Ok(json) => json,
            Err(e) => {
                println!("failed to read talent config: {:?} {:?}", e, entry.path());
                continue;
            }
        };
        // 单个文件坏掉不该拖垮整份配置：第三方数据包里出现空文件或截断文件很常见，
        // 早先这里用 `?` 直接中断循环，再被 main.rs 的 unwrap 变成 panic，
        // 服务端会在启动阶段直接挂掉，且报错不指出是哪个文件。
        match serde_json::from_slice::<TalentConfig>(&*json) {
            Ok(config) => map.extend(config.talents),
            Err(e) => {
                println!("failed to parse talent config: {:?} {:?}", e, entry.path());
            }
        }
    }

    let _ = AVATAR_TALENT_CONFIG_MAP.set(map);
    Ok(())
}

pub fn load_avatar_talent_configs_from_bin(bin_output_path: &str) -> std::io::Result<()> {
    load_avatar_talent_configs(fs::read_dir(format!(
        "{bin_output_path}/Talent/AvatarTalents/"
    ))?)?;

    Ok(())
}

pub fn get_avatar_talent_config(name: &InternString) -> Option<&Vec<TalentAction>> {
    AVATAR_TALENT_CONFIG_MAP.get().unwrap().get(name)
}

pub fn iter_avatar_talent_config_map() -> Iter<'static, InternString, Vec<TalentAction>> {
    AVATAR_TALENT_CONFIG_MAP.get().unwrap().iter()
}
pub fn process_talent_ids(
    talent_id_list: &[u32],
    avatar_talent_collection: &std::sync::Arc<HashMap<u32, AvatarTalentExcelConfig>>,
) -> Vec<InternString> {
    let mut open_configs = Vec::new();
    for talent_id in talent_id_list {
        if let Some(talent_config) = avatar_talent_collection.get(talent_id) {
            open_configs.push(talent_config.open_config);
        }
    }
    open_configs
}

pub fn process_inherent_proud_skills(
    inherent_proud_skill_list: &[u32],
    proud_skill_collection: &std::sync::Arc<HashMap<u32, ProudSkillExcelConfig>>,
) -> Vec<InternString> {
    let mut open_configs = Vec::new();
    for proud_skill_id in inherent_proud_skill_list {
        if let Some(proud_skill_config) = proud_skill_collection.get(proud_skill_id) {
            open_configs.push(proud_skill_config.open_config);
        }
    }
    open_configs
}
