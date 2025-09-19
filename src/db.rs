use mysql::*;
use mysql::prelude::*;
use std::env;


// Create a connection pool once
pub fn get_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    let db_url = "mysql://appuser:kZMHz43s3D8!!@localhost:3306/password_manager";
    let opts = Opts::from_url(db_url)?;
    let pool = Pool::new(opts)?;
    Ok(pool)
}

// Get a connection from the pool
pub fn get_conn(pool: &Pool) -> Result<PooledConn, Box<dyn std::error::Error>> {
    let conn = pool.get_conn()?;
    Ok(conn)
}

