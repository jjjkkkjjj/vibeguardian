use anyhow::Result;

use crate::cli::SetArgs;
use crate::config::secrets;

pub fn execute(args: SetArgs) -> Result<()> {
    let value = match args.value {
        Some(v) => {
            eprintln!(
                "[Vibeguard] Warning: secret value passed as a CLI argument may be visible in shell history."
            );
            v
        }
        None => {
            eprint!("Enter secret value for '{}' (hidden): ", args.path);
            rpassword::read_password()?
        }
    };

    secrets::set(&args.path, &value)?;
    println!("[Vibeguard] Secret stored at '{}'.", args.path);
    Ok(())
}
