const GENERIC_WORDS: [&str; 6] = ["microsoft", "google", "the", "app", "for", "windows"];
const MIN_WORD_LEN: usize = 3;
const PATH_SCORE: u32 = 6;
const PROCESS_SCORE: u32 = 4;
const TITLE_SCORE: u32 = 2;

/// How confidently a window belongs to a launched app, judged from the app's name or path against the
/// window's process (`winword.exe` for "Word") and title. Zero means "not this app".
pub fn match_score(app_name: &str, app_id: &str, title: &str, process: &str) -> u32 {
    let process_stem = process.trim_end_matches(".exe").to_lowercase();
    if process_stem.is_empty() {
        return 0;
    }
    if path_stem(app_id).is_some_and(|stem| stem == process_stem) {
        return PATH_SCORE;
    }
    let title = title.to_lowercase();
    let words = significant_words(app_name);
    let in_process = words.iter().any(|word| process_stem.contains(word.as_str()) || word.contains(&process_stem));
    let in_title = words.iter().any(|word| title.contains(word.as_str()));
    (if in_process { PROCESS_SCORE } else { 0 }) + (if in_title { TITLE_SCORE } else { 0 })
}

fn significant_words(name: &str) -> Vec<String> {
    name.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.chars().count() >= MIN_WORD_LEN && !GENERIC_WORDS.contains(word))
        .map(String::from)
        .collect()
}

fn path_stem(app_id: &str) -> Option<String> {
    let file = app_id.rsplit(['\\', '/']).next()?;
    let lower = file.to_lowercase();
    lower.strip_suffix(".exe").map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_apps_by_their_process() {
        assert!(match_score("Word", "Microsoft.Office.WINWORD.EXE.15", "Document1 - Word", "WINWORD.EXE") >= PROCESS_SCORE);
        assert!(match_score("Google Chrome", "Chrome", "New Tab - Google Chrome", "chrome.exe") >= PROCESS_SCORE);
        assert!(match_score("Discord", "com.squirrel.Discord.Discord", "Discord", "Discord.exe") >= PROCESS_SCORE);
        assert!(match_score("Minecraft Launcher", "Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft", "Minecraft Launcher", "MinecraftLauncher.exe") >= PROCESS_SCORE);
    }

    #[test]
    fn exact_executable_paths_win() {
        assert_eq!(match_score("game", r"C:\Games\Sky\sky.exe", "Sky", "sky.exe"), PATH_SCORE);
    }

    #[test]
    fn unrelated_windows_do_not_match() {
        assert_eq!(match_score("Google Chrome", "Chrome", "Document1 - Word", "WINWORD.EXE"), 0);
        assert_eq!(match_score("Word", "x", "Discord", "Discord.exe"), 0);
    }
}
