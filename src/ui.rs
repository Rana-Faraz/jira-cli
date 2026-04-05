use std::io::{self, IsTerminal, Write};

use anyhow::{Result, bail};

pub fn is_terminal() -> bool {
    io::stdin().is_terminal()
}

pub fn prompt_string(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush()?;

    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    let value = value.trim().to_owned();

    if value.is_empty() {
        bail!("{label} cannot be empty");
    }

    Ok(value)
}

pub fn prompt_secret(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush()?;

    let value = rpassword::read_password()?;
    let value = value.trim().to_owned();

    if value.is_empty() {
        bail!("{label} cannot be empty");
    }

    Ok(value)
}
