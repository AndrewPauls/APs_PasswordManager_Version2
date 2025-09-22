use mysql::*;
use mysql::prelude::*;

pub fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
    let url = "mysql://appuser:kZMHz43s3D8!!@localhost:3306/password_manager";
    let pool = Pool::new(url)?;
    let mut conn = pool.get_conn()?;

    let result: Vec<(String, )> = conn.query("SHOW DATABASES")?;
    println!("Databases: {:?}", result);
    Ok(())
}
