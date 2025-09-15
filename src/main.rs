use std::io;
mod create_user_data;
mod view_existing_entries;
mod add_entry;

use crate::create_user_data::LoginRecord;
use crate::add_entry::{read_json, add_record, write_json};
fn main() {
    create_user_data::create_user_data().unwrap();

    println!("Hello! This is AP's Password Management Application!");
    let string1 = "Please enter your name:"; println!("{} ", string1);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let name = input.trim();
    println!("Hello there {}!", name);

   
    while true {
        let string2 = "Would you like to \nA - View existing entries?\nB - Add a new entry?";
        let string3 = "Please enter A or B to indicate your decision.";
        println!("{} ", string2);
        println!("{} ", string3);

        let mut input2 = String::new();
        io::stdin()
            .read_line(&mut input2)
            .expect("Failed to read decision");

        let decision = input2.trim();
        println!("You have decided {}.", decision);
            if decision == "A" {
                // view existing entries function
                viewExistingEntries();
                continue;
            }
            else if decision == "B" {
                println!("\nPlease enter the data for the new entry.");
                let file_path = "PasswordRecords.json";

                let mut records = read_json(file_path).unwrap();

                let new_record = LoginRecord {
                    account_owner: "Alice".to_string(),
                    account_name: "Twitter".to_string(),
                    account_username: "@alice123".to_string(),
                    account_password: "secretkey55".to_string(),
                };

                records = add_record(records, new_record);

                //write back to json
                write_json(&records, file_path).unwrap();

                continue;
            }
            else if decision == "QUIT" {
                println!("\nExiting now.");
                break;
            }
            else {
                println!("\nFailed to detect valid decision.");
                // reroute user back to entering valid input
                continue
            }
    }

    let string4 = "Thank you for using AP's Password Manager. Powering down.";
    println!("\n{}", string4);
}

fn viewExistingEntries() {
    println!("\nPlease type in the account owners name.");
    let mut ownerInput = String::new();
    io::stdin()
        .read_line(&mut ownerInput)
        .expect("Failed to read input");

    let ownersAccountName = ownerInput.trim();
    println!("Preparing to print entries for the record owner: {}", ownersAccountName);

    // for each entry in the json file, check for that owners name, and then print the 
    // full record for each situation in which that person has a saved file
    let entries = view_existing_entries::view_entries_by_owner(ownersAccountName, "PasswordRecords.json").unwrap();

    for entry in entries {
        println!(
            "Account: {} Username: {}, Password: {}", 
            entry.account_name, entry.account_username, entry.account_password
        );
    }

}
