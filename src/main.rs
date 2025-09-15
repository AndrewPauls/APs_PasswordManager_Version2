use std::io;

fn main() {
    println!("Hello! This is AP's Password Management Application!");
    let string1 = "Please enter your name:"; println!("{} ", string1);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let name = input.trim();
    println!("Hello there {}!", name);

   
    while true {
        let string2 = "Would you like to \nA - Create an account?\nB - Login?";
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
                println!("\nBeginning Create Account Process.");
                // create account function
                break;
            }
            else if decision == "B" {
                println!("\nBeginning login Process.");
                // login function
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
