use crate::release::Asset;
use regex::Regex;

pub fn detect_architecture() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" | "i686" => "x86",
        _ => "x64",
    }
}

pub fn select<'a>(assets: &'a [Asset], pattern: &str) -> Result<&'a Asset, String> {
    select_for_arch(assets, pattern, "auto")
}

pub fn select_for_arch<'a>(
    assets: &'a [Asset],
    pattern: &str,
    architecture: &str,
) -> Result<&'a Asset, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid asset pattern: {e}"))?;
    let arch = if architecture == "auto" { detect_architecture() } else { architecture };
    let mut candidates = assets.iter().filter(|a| regex.is_match(&a.name)).collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err(format!("No release asset matches pattern: {pattern}"));
    }
    candidates.sort_by_key(|a| std::cmp::Reverse(score(&a.name, arch)));
    Ok(candidates[0])
}

fn score(name: &str, arch: &str) -> i32 {
    let n = name.to_ascii_lowercase();
    let mut value = 0;
    if n.contains("windows") || n.contains("win64") || n.contains("win-") || n.contains("win32") {
        value += 40;
    }
    if n.ends_with(".zip") { value += 8; }
    if n.ends_with(".exe") { value += 7; }
    if n.contains("portable") { value += 4; }
    if n.contains("linux") || n.contains("darwin") || n.contains("macos") || n.contains("osx") {
        value -= 100;
    }

    let is_x64 = n.contains("x86_64") || n.contains("x64") || n.contains("amd64") || n.contains("win64");
    let is_arm64 = n.contains("arm64") || n.contains("aarch64");
    let is_x86 = n.contains("x86") || n.contains("i686") || n.contains("win32") || n.contains("32-bit");
    match arch {
        "arm64" => {
            if is_arm64 { value += 45; }
            if is_x64 || is_x86 { value -= 35; }
        }
        "x86" => {
            if is_x86 && !is_x64 { value += 45; }
            if is_x64 || is_arm64 { value -= 35; }
        }
        _ => {
            if is_x64 { value += 45; }
            if is_arm64 || (is_x86 && !is_x64) { value -= 35; }
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn prefers_requested_architecture() {
        assert!(score("tool-windows-x64.zip", "x64") > score("tool-windows-arm64.zip", "x64"));
        assert!(score("tool-windows-arm64.zip", "arm64") > score("tool-windows-x64.zip", "arm64"));
        assert!(score("tool-win32.zip", "x86") > score("tool-win64.zip", "x86"));
    }
}
