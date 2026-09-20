use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snippet {
    trigger: String,
    replace: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MatchFile {
    matches: Vec<Snippet>,
}

#[derive(Debug, Serialize)]
struct GitState {
    dirty: bool,
    branch: String,
    ahead: usize,
}

fn match_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME bulunamadı.")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("espanso")
        .join("match")
        .join("base.yml"))
}

fn resolve_target(path: &Path) -> Result<PathBuf, String> {
    if path.is_symlink() {
        fs::canonicalize(path).map_err(|e| format!("Symlink çözülemedi: {e}"))
    } else {
        Ok(path.to_path_buf())
    }
}

fn read_file() -> Result<(PathBuf, MatchFile), String> {
    let path = match_path()?;
    let target = resolve_target(&path)?;

    if !target.exists() {
        return Ok((target, MatchFile::default()));
    }

    let text = fs::read_to_string(&target).map_err(|e| format!("base.yml okunamadı: {e}"))?;
    let parsed = serde_yaml::from_str::<MatchFile>(&text)
        .map_err(|e| format!("base.yml YAML olarak okunamadı: {e}"))?;
    Ok((target, parsed))
}

fn write_file(target: &Path, data: &MatchFile) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Dizin oluşturulamadı: {e}"))?;
    }

    let yaml = serde_yaml::to_string(data).map_err(|e| format!("YAML oluşturulamadı: {e}"))?;
    let parent = target.parent().ok_or("base.yml dizini bulunamadı.")?;
    let mut temp = tempfile_path(parent, target);

    {
        let mut file = fs::File::create(&temp).map_err(|e| format!("Geçici dosya açılamadı: {e}"))?;
        file.write_all(yaml.as_bytes())
            .map_err(|e| format!("Geçici dosya yazılamadı: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Geçici dosya diske yazılamadı: {e}"))?;
    }

    fs::rename(&temp, target).map_err(|e| {
        let _ = fs::remove_file(&temp);
        format!("base.yml atomik olarak değiştirilemedi: {e}")
    })?;

    temp.clear();
    Ok(())
}

fn tempfile_path(parent: &Path, target: &Path) -> PathBuf {
    let name = target.file_name().and_then(|x| x.to_str()).unwrap_or("base.yml");
    parent.join(format!(".{name}.espanso-manager.tmp"))
}

fn git_repo(target: &Path) -> Result<PathBuf, String> {
    let cwd = target.parent().ok_or("Git çalışma dizini bulunamadı.")?;
    let out = Command::new("git")
        .args(["-C", cwd.to_string_lossy().as_ref(), "rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("git çalıştırılamadı: {e}"))?;

    if !out.status.success() {
        return Err("base.yml bir Git deposunda değil.".into());
    }

    let repo = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    Ok(PathBuf::from(repo))
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| format!("git çalıştırılamadı: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn rel_path<'a>(repo: &'a Path, target: &'a Path) -> Result<String, String> {
    target
        .strip_prefix(repo)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|_| "base.yml Git deposunun dışında.".into())
}

#[tauri::command]
fn list_snippets() -> Result<Vec<Snippet>, String> {
    Ok(read_file()?.1.matches)
}

#[tauri::command]
fn save_snippet(index: Option<usize>, snippet: Snippet) -> Result<(), String> {
    let trigger = snippet.trigger.trim().to_owned();
    if trigger.is_empty() || snippet.replace.is_empty() {
        return Err("Trigger ve çıktı gerekli.".into());
    }

    let (target, mut data) = read_file()?;

    if data
        .matches
        .iter()
        .enumerate()
        .any(|(i, item)| Some(i) != index && item.trigger == trigger)
    {
        return Err("Bu trigger zaten var.".into());
    }

    let next = Snippet {
        trigger,
        replace: snippet.replace,
    };

    match index {
        Some(i) => {
            if i >= data.matches.len() {
                return Err("Snippet bulunamadı.".into());
            }
            data.matches[i] = next;
        }
        None => data.matches.push(next),
    }

    write_file(&target, &data)
}

#[tauri::command]
fn delete_snippet(index: usize) -> Result<(), String> {
    let (target, mut data) = read_file()?;
    if index >= data.matches.len() {
        return Err("Snippet bulunamadı.".into());
    }
    data.matches.remove(index);
    write_file(&target, &data)
}

#[tauri::command]
fn git_state() -> Result<GitState, String> {
    let (target, _) = read_file()?;
    let repo = git_repo(&target)?;
    let rel = rel_path(&repo, &target)?;

    let status = git_output(&repo, &["status", "--porcelain", "--", &rel])?;
    let branch = git_output(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;

    let ahead = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["rev-list", "--count", "@{u}..HEAD"])
        .output()
        .ok()
        .filter(|x| x.status.success())
        .and_then(|x| String::from_utf8(x.stdout).ok())
        .and_then(|x| x.trim().parse::<usize>().ok())
        .unwrap_or(0);

    Ok(GitState {
        dirty: !status.is_empty() || ahead > 0,
        branch,
        ahead,
    })
}

#[tauri::command]
fn git_save() -> Result<String, String> {
    let (target, _) = read_file()?;
    let repo = git_repo(&target)?;
    let rel = rel_path(&repo, &target)?;

    git_output(&repo, &["add", "--", &rel])?;

    let staged = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["diff", "--cached", "--quiet", "--", &rel])
        .status()
        .map_err(|e| format!("git diff çalıştırılamadı: {e}"))?;

    if !staged.success() {
        git_output(
            &repo,
            &["commit", "-m", "Update Espanso snippets", "--", &rel],
        )?;
    }

    git_output(&repo, &["push", "origin", "HEAD"])?;

    Ok(if staged.success() {
        "GitHub zaten günceldi.".into()
    } else {
        "Snippetler GitHub’a kaydedildi.".into()
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_snippets,
            save_snippet,
            delete_snippet,
            git_state,
            git_save
        ])
        .run(tauri::generate_context!())
        .expect("error while running espanso-manager");
}
