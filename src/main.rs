use std::io::{self, Write};
mod create_user_data;
mod view_existing_entries;
mod add_entry;
mod hashPassword;
use crate::create_user_data::LoginRecord;
use crate::add_entry::{read_json, add_record, write_json};
use crate::hashPassword::{hash_password, verify_hashed_password};

fn main() {
    // create JSON 'db' of login records
    create_user_data::create_user_data().unwrap();

    entryMessage();      // tells user how to begin using PW Manager
    beginSession();      // use the PW Manager
    exitMessage();       // close program gracefully

}

fn viewExistingEntries() {
    println!("\n\nPlease type in the account owners name to see their records.");
    let mut ownerInput = String::new();
    io::stdin()
        .read_line(&mut ownerInput)
        .expect("Failed to read input");

    let ownersAccountName = ownerInput.trim();
    println!("Preparing to print entries for the record owner: {}", ownersAccountName);

    // for each entry in the json file, check for that owners name, and then print the 
    // full record for each situation in which that person has a saved file
    let entries = view_existing_entries::view_entries_by_owner(ownersAccountName, "PasswordRecords.json").unwrap();

    // view passwords in the clear now
    println!("\nWe will now permit you to check if you know the correct passwords.");
    for entry in entries {
        println!( "Account: {} Username: {}, Password: _________", 
            entry.account_name, entry.account_username, );
        io::stdout().flush().unwrap();

        let mut attempt = String::new();
        io::stdin().read_line(&mut attempt).expect("Failed to read input");
        let attempt = attempt.trim();

        // verify now against saved json hash
        if verify_hashed_password(&entry.account_password, attempt) {
            println!("Correct Password!");
        } else {
            println!("Incorect Password.");
        }

        println!("<------------------------------------->");
    }
}

fn exitMessage() {
    let string4 = "Thank you for using AP's Password Manager. Powering down.";
    println!("\n{}", string4);
}

fn entryMessage() {
    println!("Hello! This is AP's Password Management Application!");
    let string1 = "Please enter your name:"; println!("{} ", string1);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let name = input.trim();
    println!("Hello there {}!", name);
}

fn beginSession() {
    while true {
        let decision = getDecision();
        if decision == "A" {
            // view existing entries function
            viewExistingEntries();
            continue;
        }
        else if decision == "B" {
            println!("\nPlease enter the data for the new entry.");
            let file_path = "PasswordRecords.json";
            let mut records = read_json(file_path).unwrap();
            fn prompt(msg: &str) -> String {
                print!("{}", msg);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                input.trim().to_string()
            }

            let account_owner = prompt("Enter account owner: ");
            let account_name = prompt("Enter account name: ");
            let account_username = prompt("Enter account username: ");
            let account_password = prompt("Enter account password: ");

            // hash
            let hashed_password = hash_password(&account_password);

            let new_record = LoginRecord {
                account_owner,
                account_name,
                account_username,
                account_password: hashed_password
            };

            records = add_record(records, new_record);

            //write back to json
            write_json(&records, file_path).unwrap();
            continue;
        }
        else if decision == "C" {
            println!("\nExiting now.");
            break;
        }
        else {
            println!("\nFailed to detect valid decision.");
            // reroute user back to entering valid input
            continue
        }
    }
}

fn getDecision() -> String {
    let string2 = "\n\nWould you like to \nA - View existing entries?\
                   \nB - Add a new entry?\
                   \nC - Quit";
    let string3 = "Please enter A, B, or C.";
    println!("{} ", string2);
    println!("{} ", string3);

    let mut input2 = String::new();
    io::stdin()
        .read_line(&mut input2)
        .expect("Failed to read decision");

    let decision = input2.trim();
    println!("\nYou have decided {}.", decision);

    decision.to_string()
}
