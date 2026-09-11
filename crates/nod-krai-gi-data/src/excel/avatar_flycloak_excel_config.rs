use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarFlycloakExcelConfig {
    // 7.0 起该字段名被混淆为 FOJCCHCMOIE，取值 140001 起、与滑翔翼 id 段一致。
    #[serde(alias = "FOJCCHCMOIE")]
    pub flycloak_id: u32,
    pub desc_text_map_hash: u64,
    pub name_text_map_hash: u64,
}

pub trait AvatarFlycloakExcelConfigKeyed<K> {
    fn key(&self) -> K;

    fn load(excel_bin_output_path: &str) -> HashMap<K, AvatarFlycloakExcelConfig>;
}

impl AvatarFlycloakExcelConfigKeyed<u32> for AvatarFlycloakExcelConfig {
    fn key(&self) -> u32 {
        self.flycloak_id
    }

    fn load(excel_bin_output_path: &str) -> HashMap<u32, AvatarFlycloakExcelConfig> {
        let json = std::fs::read(&format!(
            "{excel_bin_output_path}/AvatarFlycloakExcelConfigData.json"
        ))
        .unwrap();
        let list: Vec<AvatarFlycloakExcelConfig> = serde_json::from_slice(&*json).unwrap();
        let data = list.iter().map(|item| (item.key(), item.clone())).collect();
        data
    }
}
