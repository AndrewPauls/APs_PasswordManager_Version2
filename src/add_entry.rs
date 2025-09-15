use crate::create_user_data::LoginRecord;
use serde_json::Result;
use std::fs;

pub fn read_json(file_path: &str) -> Result<Vec<LoginRecord>> {
    let data = fs::read_to_string(file_path).unwrap_or_else(|_| "[]".to_string());
    let records: Vec<LoginRecord> = serde_json::from_str(&data)?;
    Ok(records)
}

pub fn add_record(mut records: Vec<LoginRecord>, new_record: LoginRecord) -> Vec<LoginRecord> {
    records.push(new_record);
    records
}

pub fn write_json(records: &Vec<LoginRecord>, file_path: &str) -> Result<()> {
    let json_string = serde_json::to_string_pretty(records)?;
    fs::write(file_path, json_string).expect("Unable to write file");
    Ok(())
}


