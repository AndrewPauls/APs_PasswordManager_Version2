use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use rpassword::read_password;
mod hashPassword;
use hashPassword::verify_hashed_password;
#[derive(Debug, Serialize, Deserialize)]
struct AddEntry {
    owner: String,
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    account_owner: Option<String>,
    account_name: Option<String>,
    account_username: Option<String>,
    account_password: Option<String>,
}

#[tokio::main]
async fn main() {
    entry_message().await;
    begin_session().await;
    exit_message().await;
}

use reqwest::Client;
use std::io::{self, Write};

#[derive(Debug, Clone)]
struct Entry {
    account_owner: Option<String>,
    account_name: Option<String>,
    account_username: Option<String>,
    account_password: Option<String>,
}

pub async fn view_existing_entries(client: &Client) {
    println!("\n\nPlease type in the account owner's name to see their records.");
    let owner = read_input();

    let url = format!("http://127.0.0.1:3000/entries/{}", owner);
    let resp = client.get(&url).send().await;

    match resp {
        Ok(response) => {
            if response.status().is_success() {
                let entries: Vec<Entry> = response.json().await.unwrap_or_else(|_| {
                    println!("Failed to parse JSON from server response.");
                    Vec::new()
                });

                display_entries(&entries);

                // Options for user
                let sel_trim = read_input();

                if sel_trim.is_empty() {
                    println!("Skipped.");
                    return;
                }

                if sel_trim.starts_with('d') {
                    handle_delete(sel_trim, &entries, client).await;
                } else {
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

fn read_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn display_entries(entries: &[Entry]) {
    if entries.is_empty() {
        println!("No entries found.");
        return;
    }

    println!("Entries for the owner:");
    for (i, entry) in entries.iter().enumerate() {
        println!("{}. Account: {}", i + 1, entry.account_name.clone().unwrap_or_default());
        println!("   Username: {}", entry.account_username.clone().unwrap_or_default());
        println!("   Password (hashed): {}", entry.account_password.clone().unwrap_or_default());
    }

    // Options for user
    println!("\nOptions:");
    println!("  Enter a number to check its password");
    println!("  Enter d<number> to delete that entry (e.g., d2 to delete entry 2)");
    println!("  Or just press Enter to skip:");
}

async fn handle_delete(sel_trim: String, entries: &[Entry], client: &Client) {
    let num_str = &sel_trim[1..];
    let sel_idx: usize = match num_str.parse::<usize>() {
        Ok(n) if n >= 1 && n <= entries.len() => n - 1,
        _ => {
            println!("Invalid selection.");
            return;
        }
    };

    let selected = &entries[sel_idx];
    let stored_hash = match &selected.account_password {
        Some(h) if !h.is_empty() => h,
        _ => {
            println!("Selected entry has no stored password hash.");
            return;
        }
    };

    // Ask user for the password before deletion
    println!("Enter the password to confirm deletion: ");
    let assumed = match read_password() {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to read password: {}", e);
            return;
        }
    };

    // Verify the entered password
    if verify_hashed_password(stored_hash, &assumed) {
        let owner = selected.account_owner.clone().unwrap_or_default();
        let name = selected.account_name.clone().unwrap_or_default();
        let url = format!("http://127.0.0.1:3000/delete/{}/{}", owner, name);

        let resp = client.delete(&url).send().await;
        match resp {
            Ok(r) => println!("Server: {}", r.text().await.unwrap_or_default()),
            Err(e) => println!("Failed to contact server: {}", e),
        }
    } else {
        println!("Incorrect password. Entry not deleted.");
    }
}

async fn handle_verification(sel_trim: String, entries: &[Entry]) {
    let sel_idx: usize = match sel_trim.parse::<usize>() {
        Ok(n) if n >= 1 && n <= entries.len() => n - 1,
        _ => {
            println!("Invalid selection.");
            return;
        }
    };

    let selected = &entries[sel_idx];
    let stored_hash = match &selected.account_password {
        Some(h) if !h.is_empty() => h,
        _ => {
            println!("Selected entry has no stored password hash.");
            return;
        }
    };

    println!("Enter the password to check: ");
    let assumed = match read_password() {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to read password: {}", e);
            return;
        }
    };

    if verify_hashed_password(stored_hash, &assumed) {
        println!("Correct password.");
    } else {
        println!("Incorrect password.");
    }
}

fn read_password() -> Result<String, std::io::Error> {
    let mut password = String::new();
    print!("Password: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut password)?;
    Ok(password.trim().to_string())
}

// This is just a placeholder, implement your actual password hashing verification.
fn verify_hashed_password(hashed: &str, password: &str) -> bool {
    // Your password hash comparison logic here.
    hashed == password
}

async fn add_new_entry(client: &Client) {
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

    let hashed_password = hashPassword::hash_password(&account_password);
    let new_entry = AddEntry {
        owner: account_owner,
        name: account_name,
        username: account_username,
        password: hashed_password,
    };

    let resp = client.post("http://127.0.0.1:3000/add")
        .json(&new_entry)
        .send()
        .await;

    match resp {
        Ok(response) => {
            if response.status().is_success() {
                println!("\nSuccessfully added new entry.");
                println!("  Owner: {}", new_entry.owner);
                println!("  Name: {}", new_entry.name);
                println!("  Username: {}", new_entry.username);
                println!("  Password: {}", account_password);
            } else {
                println!("Failed to add entry. Server status: {}", response.status());
            }
        }
        Err(e) => {
            println!("Failed to contact server: {}", e);
        }
    }
}

async fn begin_session() {
    let client = reqwest::Client::new();

    loop {
        let decision = get_decision().await;
        match decision.as_str() {
            "A" => view_existing_entries(&client).await,
            "B" => add_new_entry(&client).await,
            "C" => {
                println!("\nExiting now.");
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

