use crate::{config::{self, Language}, plan, profiles, update_v13};
use std::{fs, path::PathBuf, sync::{Arc, Mutex}};

pub fn maybe_run(args: &[String]) -> Option<i32> {
    let profile = arg_value(args, "--scheduled-profile")?;
    let mode = arg_value(args, "--scheduled-mode").unwrap_or_else(|| "check".into());
    Some(match run(&profile, &mode) { Ok(()) => 0, Err(_) => 1 })
}

fn run(profile_name: &str, mode: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let base = exe.parent().map(PathBuf::from).ok_or_else(|| "Executable has no parent".to_string())?;
    let profiles_path = profiles::default_path(&base);
    let list = profiles::load(&profiles_path).map_err(|e| e.to_string())?;
    let profile = list.into_iter().find(|p| p.name.eq_ignore_ascii_case(profile_name)).ok_or_else(|| format!("Profile not found: {profile_name}"))?;
    let mut config = profile.to_config(Language::En, String::new());
    config.repo = config::normalize_repo(&config.repo)?;
    let result = if mode.eq_ignore_ascii_case("update") {
        update_v13::run(&config, &Arc::new(Mutex::new(String::new())))
    } else {
        plan::dry_run(&config)
    };
    let log_dir = base.join(".distronomicon").join("scheduled");
    fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    let log = log_dir.join(format!("{}.log", profile_name.replace(['/', '\\', ':'], "_")));
    let text = match &result { Ok(value) => format!("OK\n{value}\n"), Err(error) => format!("ERROR\n{error}\n") };
    fs::write(log, text).map_err(|e| e.to_string())?;
    result.map(|_| ())
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|pair| pair[0] == key).map(|pair| pair[1].clone())
}
