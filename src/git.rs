use std::collections::HashMap;
use std::path::{Path, PathBuf};
use git2::{BlameOptions, Branch, BranchType, Config, Oid, Repository};

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("No default branch found")]
    NoBranch,
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

type Result<T> = std::result::Result<T, GitError>;

pub fn get_file_info(path: &Path) -> Result<(usize, HashMap<String, usize>)> {
    let repo = Repository::discover(path)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Other("Not a git repository".into()))?;

    let relative_path = get_relative_path(path, workdir)?;

    // Check if the file is ignored or doesn't exist in the repo
    if repo.is_path_ignored(&relative_path)?
        || !file_exists_in_repo(&repo, &relative_path, None)?
    {
        let current_user = get_current_user(&repo)?;
        let lines = std::fs::read_to_string(path)?.lines().count();
        let mut contributors = HashMap::new();
        contributors.insert(current_user, lines);
        return Ok((lines, contributors));
    }

    let default_branch = get_default_branch(&repo)?;
    let blame = blame_file(&repo, &relative_path, default_branch.get().target())?;

    Ok(process_blame(blame))
}

fn get_relative_path(path: &Path, workdir: &Path) -> Result<PathBuf> {
    path.strip_prefix(workdir)
        .or_else(|_| path.strip_prefix("."))
        .map(PathBuf::from)
        .map_err(|e| GitError::Other(format!("Failed to create relative path: {}", e)))
}

fn file_exists_in_repo(repo: &Repository, path: &Path, target: Option<Oid>) -> Result<bool> {
    let tree = if let Some(oid) = target {
        repo.find_commit(oid)?.tree()?
    } else {
        repo.head()?.peel_to_tree()?
    };

    Ok(tree.get_path(path).is_ok())
}

fn blame_file<'a>(
    repo: &'a Repository,
    path: &Path,
    target: Option<Oid>,
) -> Result<git2::Blame<'a>> {
    let mut opts = BlameOptions::new();
    if let Some(oid) = target {
        opts.newest_commit(oid);
    }
    repo.blame_file(path, Some(&mut opts))
        .map_err(GitError::from)
}

fn process_blame(blame: git2::Blame) -> (usize, HashMap<String, usize>) {
    blame
        .iter()
        .fold((0, HashMap::new()), |(lines, mut contributors), hunk| {
            let new_lines = lines + hunk.lines_in_hunk();
            let name = hunk
                .final_signature()
                .name()
                .unwrap_or("Unknown")
                .to_string();
            *contributors.entry(name).or_insert(0) += hunk.lines_in_hunk();
            (new_lines, contributors)
        })
}

fn get_default_branch(repo: &Repository) -> Result<Branch> {
    repo.head()
        .map(Branch::wrap)
        .or_else(|_| find_head_branch(repo))
}

fn find_head_branch(repo: &Repository) -> Result<Branch> {
    repo.branches(Some(BranchType::Local))?
        .filter_map(|b| b.ok())
        .find(|(branch, _)| branch.is_head())
        .map(|(branch, _)| branch)
        .ok_or(GitError::NoBranch)
}

fn get_current_user(repo: &Repository) -> Result<String> {
    let config = repo.config()?;
    let name = config
        .get_string("user.name")
        .or_else(|_| {
            Config::open_default().and_then(|config| config.get_string("user.name"))
        })
        .or_else(|_| std::env::var("USER").or_else(|_| std::env::var("USERNAME")))
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(name)
}
