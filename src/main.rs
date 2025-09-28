use axum::{                             // Axum - an up-to-date web framework for Rust
    extract::{Path, State},             
    routing::{delete, get, post},
    Json, Router,
};
use dotenvy::dotenv;                    // dotenvy - loads env variables from .env
use hyper::Server;                      // hyper - Server: a fast HTTP implementation
use serde::{Deserialize, Serialize};    // serde - framework for serializing and deserializing Rust data structures 
use sqlx::mysql::MySqlPoolOptions;      // sqlx - async, pure Rust SQL crate
use std::io::{self, Write};             // std::io - for input/output operations
use reqwest::Client;                    // reqwest - HTTP Client for making requests
use rpassword::read_password;           // Import the `read_password` function from the `rpassword` crate
mod hashPassword;                       // Import of local `hashPassword` module
use hashPassword::verify_hashed_password; // Import the `verify_hashed_password` function from your local `hashPassword` module

#[derive(Deserialize)]
struct ApiResponse {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]    // Struct for adding a new entry
struct AddEntry {
    owner: String,
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]    // Struct for viewing existing entries
struct Entry {
    account_owner: Option<String>,
    account_name: Option<String>,
    account_username: Option<String>,
    account_password: Option<String>,
}

#[tokio::main]                      // Main function using Tokio runtime for async operations
async fn main() {
    entry_message().await;      // await, pause until future finishes
    begin_session().await;
    exit_message().await;
}

/* view_existing_entries
Accepts: client
Returns: ()
Actions:
    1. Instructs user to enter the records of the account owner they wish to view
    2. Creates URL as formatted String, sends URL as request (GET), acquires response from server
    3. Checks Response, prints account owner's records if valid
Error Handling: 
    1. let entries: Vec<Entry> If fails to unwrap, return empty vec, print error
    2. if response.status != success (HTTP response in 2** range), return response from server
    3. If bad server response, print failed to contact server, error {Ok, Err} 
*/
pub async fn view_existing_entries(client: &Client) {
    println!("\n\nPlease type in the account owner's name to see their records.");
    let owner = read_input();

    let url = format!("http://127.0.0.1:3000/entries/{}", owner);
    let resp = client.get(&url).send().await; // Send GET request to server

    match resp {
        Ok(response) => {
            if response.status().is_success() {
                let entries: Vec<Entry> = response.json().await.unwrap_or_else(|_| {    // Parse JSON response (Vec<Entry>)
                    println!("Failed to parse JSON from server response.");
                    Vec::new()
                });

                display_entries(&entries);  // display the password records for indicated owner

                // Options for user (grant them further options of deleting a record, checking a password)
                let sel_trim = read_input();

                if sel_trim.is_empty() {
                    println!("Skipped.");
                    return;
                }

                if sel_trim.starts_with('d') {  // they want to delete an entry
                    handle_delete(sel_trim, &entries, client).await;
                } else {    // they want to check a password (or made a mistake)
                    handle_verification(sel_trim, &entries).await;
                }
            } else {
                println!("Server responded with status: {}", response.status());
            }
        }
        Err(e) => {
            println!("Failed to contact server: {}", e);
        }
    }
}

/* read_input
Accepts: Nothing
Returns: String
Actions:
    1. initialize string (input)
    2. read standard input from user
    3. returns trimmed string
Error Handling:
    1. Expect error in reading input, tell user bad read
    2. Error with usize
*/
fn read_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

/* display_entries
Accepts: slice of entries 
Returns: ()
Actions:
    1. Prints no entries of [Entry] is empty
    2. Prints each entry if there are entries
    3. Offers user additional manoevers once entries are displayed
Error Handling:
    1. Empty entries
*/
fn display_entries(entries: &[Entry]) {
    if entries.is_empty() {
        println!("No entries found.");
        return;
    }

    println!("Entries for the owner:");
    for (i, entry) in entries.iter().enumerate() {  // create iterator over entries, enumerates to reference index
        println!("{}. Account: {}", i + 1, entry.account_name.clone().unwrap_or_default()); // index + 1, don't start at 0 start at 1
        println!("   Username: {}", entry.account_username.clone().unwrap_or_default());
        println!("   Password (hashed): {}", entry.account_password.clone().unwrap_or_default());
    }

    // Options for user
    println!("\nOptions:");
    println!("  Enter a number to check its password");
    println!("  Enter d<number> to delete that entry (e.g., d2 to delete entry 2)");
    println!("  Or just press Enter to skip:");
}

/* handle_delete
Accepts: 
    1. String = decision from user about which entry to delete
    2. Slice of Entry's = the available entries for the account owner
    3. Client = the users session
Returns: ()
Actions: 
    1.  Trim user input to attain the index of the record to be deleted
    2.  Acquire the record to be deleted
    3.  Check if selected record has a hashed password
    4.  Get password for account from user
    5.  Check if the passwords match
    6.  If valid password, 'delete' HTTP request sent
Error Handling:
    1.  Bad input from user regarding d<#> to be deleted
    2.  No saved password hash for record
    3.  Bad password read from user
    4.  Failed to contact server
    5.  Incorrect password
*/
async fn handle_delete(sel_trim: String, entries: &[Entry], client: &Client) {
    let num_str = &sel_trim[1..];       // trim to index 1 and beyond
    let sel_idx: usize = match num_str.parse::<usize>() {   // parse string to int
        Ok(n) if n >= 1 && n <= entries.len() => n - 1, // check input is >=1, <= len
        _ => {  // if not, print invalid selection
            println!("Invalid selection.");
            return;
        }
    };

    let selected = &entries[sel_idx];   // selected = entry to be
    let stored_hash = match &selected.account_password {    // get entry's stored hash
        Some(h) if !h.is_empty() => h,      // if there is something, return it
        _ => {  // nothing to return (no stored hash)
            println!("Selected entry has no stored password hash.");
            return;
        }
    };

    // Ask user for the password before deletion
    println!("Enter the password to confirm deletion: ");
    let assumed = match read_password() {   // assumed = standard input password attempt
        Ok(s) => s, // read string from user
        Err(e) => {     // error reading password
            println!("Failed to read password: {}", e);
            return;
        }
    };

    // Verify the entered password
    if verify_hashed_password(stored_hash, &assumed) {  // if user enters correct password
        let owner = selected.account_owner.clone().unwrap_or_default(); // separate 'owner' for url string
        let name = selected.account_name.clone().unwrap_or_default();   // separate 'account name' for url
        let url = format!("http://127.0.0.1:3000/delete/{}/{}", owner, name);   // generate url, to be used to delete (a delete request)

        let resp = client.delete(&url).send().await;    // acquire response from cleint after attempting delete operation
        match resp {
            Ok(r) => {
                let body_text = r.text().await.unwrap_or_default(); // read body once

                match serde_json::from_str::<ApiResponse>(&body_text) {
                    Ok(api_resp) => println!("Server: {}", api_resp.message),
                    Err(e) => {
                        println!("Failed to parse server response: {}", e);
                        println!("Raw response: {}", body_text);
                    }
                }
            }
            Err(e) => println!("Failed to contact server: {}", e),
        }
    } else {
        println!("Incorrect password. Entry not deleted.");
    }
}

/* handle_verification
Accepts: 
    1. String - index of password to be verified
    2. entries: slice of account owners entries
Returns: ()
Actions: 
    1.  Gather desired index of record to be verified
    2.  If desired record has stored hash, user enters assumed password
    3.  Passwords checked
    4.  If passwords match, indicate they match
Error Handling: 
    1.  Invalid index of record to be checked
    2.  No saved password for indicated record
    3.  Bad Standard Input read
    4.  Incorrect password during verifcation
*/
async fn handle_verification(sel_trim: String, entries: &[Entry]) {
    let sel_idx: usize = match sel_trim.parse::<usize>() {
        Ok(n) if n >= 1 && n <= entries.len() => n - 1, // check input is >=1, <= len
        _ => {
            println!("Invalid selection.");
            return;
        }
    };

    let selected = &entries[sel_idx];   // get selected entry
    let stored_hash = match &selected.account_password {    // get saved hash
        Some(h) if !h.is_empty() => h,  // if there is a hash, return it
        _ => {
            println!("Selected entry has no stored password hash.");
            return;
        }
    };

    println!("Enter the password to check: ");  
    let assumed = match read_password() {       // get assumed password from user
        Ok(s) => s,
        Err(e) => { // couldn't get the password during read
            println!("Failed to read password: {}", e);
            return;
        }
    };

    if verify_hashed_password(stored_hash, &assumed) {  // if passwords match
        println!("Correct password.");
    } else {
        println!("Incorrect password.");
    }
}

/* add_new_entry
Accepts: Client
Returns: ()
Actions: 
    1. User enters data for new record to be added
    2. Password is hashed
    3. New entry object created (AddEntry struct)
    4. New entry added to DB via a post
    5. When successfully added, results shown to user
Error Handling:
    1.  Bad input read
    2.  Unable to add entry
    3.  Unable to connect to server
*/
async fn add_new_entry(client: &Client) {
    println!("\nPlease enter the data for the new entry.");

    // Helper function to prompt user and read input
    fn prompt(msg: &str) -> String {
        print!("{}", msg);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().to_string()
    }

    // Gather data for new entry
    let account_owner = prompt("Enter account owner: ");
    let account_name = prompt("Enter account name: ");
    let account_username = prompt("Enter account username: ");
    let account_password = prompt("Enter account password: ");

    let hashed_password = hashPassword::hash_password(&account_password);
    let new_entry = AddEntry {  // create object for adding to DB
        owner: account_owner,
        name: account_name,
        username: account_username,
        password: hashed_password,
    };

    // resp = (Response, Error) after client posts 'add' with the new entry as JSON
    let resp = client.post("http://127.0.0.1:3000/add")
        .json(&new_entry)       // serialize new_entry to JSON
        .send() // send the request
        .await;

    match resp {
        Ok(response) => {
            if response.status().is_success() {
               match response.json::<ApiResponse>().await {
                   Ok(api_resp) => println!("{}", api_resp.message),
                   Err(e) => println!("Failed to parse API response: {}", e),
               } 
            } else {
                   println!("Server error: {}", response.status());
            }
        }
        Err(e) => {
            println!("Failed to contact server: {}", e);
        }
    }
}

/* begin_session
Accepts: Nothing
Returns: ()
Actions:
    1. Creates client, to be used for making HTTP requests
    2. Gets decision from user about what to do
    3. Handles decision
    4. Repeat
*/
async fn begin_session() {
    let client = reqwest::Client::new();    // a reqwest object for making HTTP requests during the users session

    loop {      // until user exist, perform loop
        let decision = get_decision().await;    // the users decision about what to do
        match decision.as_str() {   // based on decision... do _____
            "A" => view_existing_entries(&client).await, // client passed to potentially make request
            "B" => add_new_entry(&client).await,            // client passed to potentially make request
            "C" => {
                println!("\nExiting now.");     // session deemed done
                break;
            }
            _ => println!("\nInvalid choice, please try again."),
        }
    }
}

async fn get_decision() -> String {
    let prompt = "\n\nWould you like to \nA - View existing entries?\nB - Add a new entry?\nC - Quit";
    let reminder = "Please enter A, B, or C.";
    println!("{}", prompt);
    println!("{}", reminder);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read decision");

    input.trim().to_uppercase()
}

async fn entry_message() {
    println!("Hello! This is AP's Password Management Application!");
    println!("Please enter your name:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let name = input.trim();
    println!("Hello there {}!", name);
}

async fn exit_message() {
    println!("\nThank you for using AP's Password Manager. Powering down.");
}
