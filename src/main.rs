use std::io::{self, Write};
use mysql::prelude::*;
use mysql::{Opts, Pool};
use std::env;
use mysql::params;
mod create_user_data;
mod view_existing_entries;
mod add_entry;
mod hashPassword;
mod db;
use crate::create_user_data::LoginRecord;
use crate::add_entry::{read_json, add_record, write_json};
use crate::hashPassword::{hash_password, verify_hashed_password};
use std::fs;
mod db_test;

fn main() {
    // create JSON 'db' of login records
    create_user_data::create_user_data().unwrap();

    // test db conn
    match db_test::test_connection() {
        Ok(_) => { println!("Connection successful!"); }
        Err(e) => { eprintln!("Connection failed: {}", e); }
    }
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

    let pool = db::get_pool().unwrap();
    let mut conn = db::get_conn(&pool).unwrap();

    let entries: Vec<LoginRecord> = conn.exec_map(
        "SELECT account_owner, account_name, account_username, account_password
        FROM password_records WHERE account_owner = :owner",
        params! {
            "owner" => ownersAccountName,
        },
        |(account_owner, account_name, account_username, account_password) | {
            LoginRecord {
                account_owner,
                account_name,
                account_username,
                account_password,
            }
        },
    ).expect("Failed to fetch records");        
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

            let pool = db::get_pool().unwrap();
            let mut conn = db::get_conn(&pool).unwrap();

            conn.exec_drop(
                "INSERT INTO password_records (account_owner, account_name, account_username, account_password)
                VALUES (:owner, :name, :username, :password)",
                params! {
                    "owner" => &new_record.account_owner,
                    "name" => &new_record.account_name,
                    "username" => &new_record.account_username,
                    "password" => &new_record.account_password,
                }
            ).expect("Failed to insert record");

            continue
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
