use anyhow::Result;

use crate::cli::SetArgs;
use crate::config::{project::ProjectConfig, secrets};

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

    if args.project {
        let config = ProjectConfig::load()?;
        let project_name = &config.project.name;
        secrets::set_project(project_name, &args.path, &value)?;
        println!(
            "[Vibeguard] Secret stored at '{}' (project: {}).",
            args.path, project_name
        );
    } else {
        secrets::set(&args.path, &value)?;
        println!("[Vibeguard] Secret stored at '{}'.", args.path);
    }

    Ok(())
}
