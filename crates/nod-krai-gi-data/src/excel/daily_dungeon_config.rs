use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
// 7.0 起周一至周日七个字段名全部被混淆。数据里有 10 个同构的数组字段，
// 无法可靠对应到具体星期，因此暂时全部缺省为空——每日秘境轮换会失效，
// 需要拿到该版本的字段名映射后再补。
pub struct DailyDungeonConfig {
    pub id: u32,
    #[serde(default)]
    pub monday: Vec<u32>,
    #[serde(default)]
    pub tuesday: Vec<u32>,
    #[serde(default)]
    pub wednesday: Vec<u32>,
    #[serde(default)]
    pub thursday: Vec<u32>,
    #[serde(default)]
    pub friday: Vec<u32>,
    #[serde(default)]
    pub saturday: Vec<u32>,
    #[serde(default)]
    pub sunday: Vec<u32>,
}

pub trait DailyDungeonConfigKeyed<K> {
    fn key(&self) -> K;

    fn load(excel_bin_output_path: &str) -> HashMap<K, DailyDungeonConfig>;
}

impl DailyDungeonConfigKeyed<u32> for DailyDungeonConfig {
    fn key(&self) -> u32 {
        self.id
    }

    fn load(excel_bin_output_path: &str) -> HashMap<u32, DailyDungeonConfig> {
        let json = std::fs::read(&format!(
            "{excel_bin_output_path}/DailyDungeonConfigData.json"
        ))
        .unwrap();
        let list: Vec<DailyDungeonConfig> = serde_json::from_slice(&*json).unwrap();
        let data = list.iter().map(|item| (item.key(), item.clone())).collect();
        data
    }
}
