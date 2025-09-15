use serde::{Serialize, Deserialize};
use serde_json::Result;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRecord {
    pub account_owner: String,
    pub account_name: String,
    pub account_username: String,
    pub account_password: String,
}

pub fn create_user_data() -> Result<()> {
    let record1 = LoginRecord {
        account_owner: "Andrew".to_string(),
        account_name: "Gmail".to_string(),
        account_username: "bitsbugsbites@gmail.com".to_string(),
        account_password: "blueFLAMINGO".to_string(),
    };

    let record2 = LoginRecord {
        account_owner: "Andrew".to_string(),
        account_name: "Bank".to_string(),
        account_username: "3452331".to_string(),
        account_password: "83dK$#d)".to_string(),
    };

    let record3 = LoginRecord {
        account_owner: "Roy".to_string(),
        account_name: "Instagram".to_string(),
        account_username: "WesternTrain".to_string(),
        account_password: "8822FortyFour".to_string(),
    };


    // gather structs into one
    let records = vec![record1, record2, record3];


    // Serialize struct into JSON
    let json_string = serde_json::to_string_pretty(&records)?;

    // Write JSON file
    let mut file = File::create("PasswordRecords.json").expect("Could not create file");
    file.write_all(json_string.as_bytes()).expect("Could not write to file");

    println!("PasswordRecords.json has been created!");
    Ok(())
}
