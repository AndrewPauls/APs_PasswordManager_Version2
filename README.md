~ ~ ~ APs Password Manager ~ ~ ~

A fun and simple password manager project in Rust and Mysql.
Upon download, users must install Mysql on their local machine and configure a Mysql
database with the following table:

Account Owner:
Account:
Username:
Password:

They must then create a .env file in the project root, in which they build their own Database URL with their login credentials to Mysql. After building the Mysql table, launching Mysql, and inserting the correct .env file, the password manager will work by 
1) Running /password_api/src/main.rs with "cargo build"
2) Running /src/main.rs with "cargo build"


The application is terminal based and offers the adding of passwords/usernames for individual owners. Users can view the records saved for an individual user, but cannot see the cleartext password as it is hashed with argon2 before being saved. Users can delete password records. 



