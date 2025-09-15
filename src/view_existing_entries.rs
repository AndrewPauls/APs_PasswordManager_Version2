use crate::create_user_data::LoginRecord;
use serde_json::Result;
use std::fs;

pub fn view_entries_by_owner(owner_name: &str, file_path: &str) -> Result<Vec<LoginRecord>> {
    //read JSON
    let data = fs::read_to_string(file_path).expect("Cant read file");

    // parse from json into Vec<LoginRecord>
    let records: Vec<LoginRecord> = serde_json::from_str(&data)?;

    // filter, owner
    let filtered: Vec<LoginRecord> = records
        .into_iter()
        .filter(|r| r.account_owner == owner_name)
        .collect();

    Ok(filtered)
}
