use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnecdoteExcelConfig {
    // 7.0 起该字段名被混淆为 KKNGMIGLAOM；取值仍是 100001/100002/100101 这类
    // anecdote id，其余候选字段都是 hash 状随机值，据此判定。
    #[serde(alias = "KKNGMIGLAOM")]
    pub anecdote_id: u32,
    // 7.0 数据里此字段同样被混淆，且有多个同构数组字段无法区分，暂缺省为空。
    #[serde(default, alias = "anecdoteQuestId")]
    pub parent_quest_id_list: Vec<u32>,
}

pub trait AnecdoteExcelConfigKeyed<K> {
    fn key(&self) -> K;

    fn load(excel_bin_output_path: &str) -> HashMap<K, AnecdoteExcelConfig>;
}

impl AnecdoteExcelConfigKeyed<u32> for AnecdoteExcelConfig {
    fn key(&self) -> u32 {
        self.anecdote_id
    }

    fn load(excel_bin_output_path: &str) -> HashMap<u32, AnecdoteExcelConfig> {
        let json = std::fs::read(&format!(
            "{excel_bin_output_path}/AnecdoteExcelConfigData.json"
        ))
        .unwrap();
        let list: Vec<AnecdoteExcelConfig> = serde_json::from_slice(&*json).unwrap();
        let data = list.iter().map(|item| (item.key(), item.clone())).collect();
        data
    }
}
