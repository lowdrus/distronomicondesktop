use crate::release::Asset;
use regex::Regex;

pub fn select<'a>(assets: &'a [Asset], pattern: &str) -> Result<&'a Asset, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid asset pattern: {e}"))?;
    let mut candidates = assets.iter().filter(|a| regex.is_match(&a.name)).collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err(format!("No release asset matches pattern: {pattern}"));
    }
    candidates.sort_by_key(|a| std::cmp::Reverse(score(&a.name)));
    Ok(candidates[0])
}

fn score(name: &str) -> i32 {
    let n = name.to_ascii_lowercase();
    let mut value = 0;
    if n.contains("windows") || n.contains("win64") || n.contains("win-") { value += 40; }
    if n.contains("x86_64") || n.contains("x64") || n.contains("amd64") { value += 30; }
    if n.ends_with(".zip") { value += 8; }
    if n.ends_with(".exe") { value += 7; }
    if n.contains("portable") { value += 4; }
    if n.contains("linux") || n.contains("darwin") || n.contains("macos") || n.contains("osx") { value -= 100; }
    if n.contains("arm64") || n.contains("aarch64") || n.contains("i686") { value -= 25; }
    value
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn prefers_windows_x64() {
        assert!(score("tool-windows-x64.zip") > score("tool-linux-x64.zip"));
        assert!(score("tool-win64.exe") > score("tool-windows-arm64.zip"));
    }
}
