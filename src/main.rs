use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

mod cmd;
mod git;
mod languages;
mod tui;

#[derive(Debug, Clone, Default)]
pub struct ContributorInfo {
    pub lines: usize,
    pub files: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LanguageInfo {
    pub lines: usize,
    pub files: usize,
}

#[derive(Debug, Default)]
pub struct Stats {
    pub languages: HashMap<String, LanguageInfo>,
    pub contributors: HashMap<String, ContributorInfo>,
}

fn main() -> Result<(), Box<dyn Error>> {
    cmd::execute()
}

pub fn get_stats(path: &str) -> Result<Stats, Box<dyn Error>> {
    let mut stats = Stats::default();
    process_directory(Path::new(path), &mut stats)?;
    Ok(stats)
}

fn process_directory(dir: &Path, stats: &mut Stats) -> Result<(), Box<dyn Error>> {
    if !dir.is_dir() {
        return Err(format!("The path '{}' is not a directory.", dir.display()).into());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_valid_file(&path) {
            process_file(&path, stats)?;
        } else if path.is_dir() {
            process_directory(&path, stats)?;
        }
    }
    Ok(())
}

fn process_file(path: &Path, stats: &mut Stats) -> Result<(), Box<dyn Error>> {
    if let Some(language) = get_language_name(path) {
        let content = fs::read_to_string(path)?;
        let lines = content.lines().count();

        let lang_info = stats.languages.entry(language.clone()).or_default();
        lang_info.lines += lines;
        lang_info.files += 1;

        match git::get_file_info(path) {
            Ok((_, file_contributors)) => {
                for (contributor, contributor_lines) in file_contributors {
                    update_contributor_info(&mut stats.contributors, &contributor, contributor_lines);
                }
            }
            Err(e) => {
                eprintln!("Git error for file {}: {}", path.display(), e);
                update_contributor_info(&mut stats.contributors, "Unknown", lines);
            }
        }
    }
    Ok(())
}

fn update_contributor_info(
    contributors: &mut HashMap<String, ContributorInfo>,
    contributor: &str,
    lines: usize,
) {
    let contrib_info = contributors.entry(contributor.to_string()).or_default();
    contrib_info.lines += lines;
    contrib_info.files += 1;
}

fn is_valid_file(path: &Path) -> bool {
    path.extension().is_some()
}

fn get_language_name(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| languages::get_language_name(ext))
}
