use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutFile {
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_processes")]
    pub process: Vec<String>,
    #[serde(default)]
    pub title_pattern: Option<String>,
    #[serde(default)]
    pub title_contains: Option<String>,
    #[serde(alias = "shortcuts")]
    pub references: Vec<ReferenceGroup>,
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceGroup {
    pub group: String,
    pub items: Vec<ReferenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceItem {
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub keys: Vec<String>,
    pub label: String,
    #[serde(default)]
    #[serde(alias = "command")]
    pub value: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub search_terms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub process_name: String,
    pub window_title: String,
    pub process_candidates: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PanelData {
    pub app_name: String,
    pub groups: Vec<ReferenceGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PickerApp {
    pub id: String,
    pub name: String,
    pub processes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LookupTrace {
    pub panel_data: Option<PanelData>,
    pub log_lines: Vec<String>,
}

pub fn scan_shortcuts(dir: &Path) -> HashMap<String, ShortcutFile> {
    let mut map = HashMap::new();

    if !dir.exists() {
        fs::create_dir_all(dir).expect("Failed to create references directory");
        return map;
    }

    for entry in walkdir(dir) {
        let Ok(path) = entry else {
            continue;
        };

        if !is_yaml_file(&path) {
            continue;
        }

        if let Some(file) = parse_shortcut_file(&path) {
            if file.process.is_empty() {
                map.insert(manual_only_identity(&file), file);
                continue;
            }

            for process_name in &file.process {
                map.insert(process_name.to_lowercase(), file.clone());
            }
        }
    }

    map
}

pub fn lookup_shortcuts_with_trace(
    map: &HashMap<String, ShortcutFile>,
    window: &WindowInfo,
) -> LookupTrace {
    let mut log_lines = vec![format!(
        "reference lookup: active process='{}' candidates={:?} title='{}'",
        window.process_name, window.process_candidates, window.window_title
    )];

    let mut direct_match_found = false;
    for candidate in &window.process_candidates {
        if let Some(file) = map.get(candidate) {
            direct_match_found = true;
            let title_match = title_pattern_matches(file, &window.window_title);
            log_lines.push(format!(
                "direct process match: '{}' -> {} [{}]",
                candidate,
                describe_file(file),
                describe_title_check(file, title_match),
            ));

            if title_match {
                log_lines.push(format!(
                    "selected {} because process matched '{}' and title check passed",
                    describe_file(file),
                    candidate,
                ));

                return LookupTrace {
                    panel_data: Some(panel_data_for(file)),
                    log_lines,
                };
            }
        } else {
            log_lines.push(format!("no direct process match for '{}'", candidate));
        }
    }

    if !direct_match_found && window.process_candidates.is_empty() {
        log_lines.push("no process candidates were available for direct matching".to_string());
    }

    for file in unique_files(map)
        .into_iter()
        .filter(|file| !file.process.is_empty())
    {
        let title_match = title_pattern_matches(file, &window.window_title);
        log_lines.push(format!(
            "fallback candidate: {} [{}]",
            describe_file(file),
            describe_title_check(file, title_match),
        ));

        if title_match {
            log_lines.push(format!(
                "selected {} because its title pattern matched the active window",
                describe_file(file),
            ));

            return LookupTrace {
                panel_data: Some(panel_data_for(file)),
                log_lines,
            };
        }
    }

    log_lines.push("no reference file matched the active window".to_string());

    LookupTrace {
        panel_data: None,
        log_lines,
    }
}

pub fn get_picker_apps(map: &HashMap<String, ShortcutFile>) -> Vec<PickerApp> {
    let mut apps = Vec::new();

    for file in unique_files(map) {
        apps.push(PickerApp {
            id: file_identity(file),
            name: file.name.clone(),
            processes: file.process.clone(),
        });
    }

    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    apps
}

pub fn lookup_picker_app(
    map: &HashMap<String, ShortcutFile>,
    picker_id: &str,
) -> Option<PanelData> {
    unique_files(map)
        .into_iter()
        .find(|file| file_identity(file) == picker_id)
        .map(panel_data_for)
}

fn unique_files<'a>(map: &'a HashMap<String, ShortcutFile>) -> Vec<&'a ShortcutFile> {
    let mut keyed_files: Vec<(String, &'a ShortcutFile)> = map
        .values()
        .map(|file| (file_identity(file), file))
        .collect();

    keyed_files.sort_by(|left, right| left.0.cmp(&right.0));
    keyed_files.dedup_by(|left, right| left.0 == right.0);

    keyed_files.into_iter().map(|(_, file)| file).collect()
}

fn file_identity(file: &ShortcutFile) -> String {
    file.source_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{}::{:?}", file.name, file.process))
}

fn manual_only_identity(file: &ShortcutFile) -> String {
    format!("__manual__::{}", file_identity(file))
}

fn panel_data_for(file: &ShortcutFile) -> PanelData {
    PanelData {
        app_name: file.name.clone(),
        groups: file.references.clone(),
    }
}

fn describe_file(file: &ShortcutFile) -> String {
    let path = file
        .source_path
        .as_ref()
        .map(|source_path| source_path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<unknown path>".to_string());

    format!("'{}' ({path})", file.name)
}

fn describe_title_check(file: &ShortcutFile, matched: bool) -> String {
    let pattern = match &file.title_pattern {
        Some(pattern) => format!("title_pattern={pattern:?}"),
        None => "title_pattern=<none>".to_string(),
    };
    let contains = match &file.title_contains {
        Some(value) => format!("title_contains={value:?}"),
        None => "title_contains=<none>".to_string(),
    };

    format!("{pattern} {contains} matched={matched}")
}

impl WindowInfo {
    pub fn from_process_name(process_name: impl Into<String>, window_title: impl Into<String>) -> Self {
        let process_name = process_name.into();
        let process_candidates = normalized_process_candidates([process_name.as_str()]);

        Self {
            process_name,
            window_title: window_title.into(),
            process_candidates,
        }
    }

    pub fn from_candidates(
        process_name: impl Into<String>,
        window_title: impl Into<String>,
        raw_candidates: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        let process_name = process_name.into();
        let mut candidate_inputs = vec![process_name.clone()];
        candidate_inputs.extend(
            raw_candidates
                .into_iter()
                .map(|value| value.as_ref().to_string())
                .filter(|value| !value.trim().is_empty()),
        );

        Self {
            process_name,
            window_title: window_title.into(),
            process_candidates: normalized_process_candidates(candidate_inputs.iter().map(String::as_str)),
        }
    }
}

fn normalized_process_candidates<'a>(
    raw_candidates: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let mut candidates = Vec::new();

    for raw_value in raw_candidates {
        for variant in identifier_variants(raw_value) {
            if !candidates.contains(&variant) {
                candidates.push(variant);
            }
        }
    }

    candidates
}

fn identifier_variants(raw_value: &str) -> Vec<String> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut variants = Vec::new();
    let lower = trimmed.to_lowercase();
    push_variant(&mut variants, &lower);

    if let Some(file_name) = std::path::Path::new(trimmed)
        .file_name()
        .and_then(|value| value.to_str())
    {
        push_variant(&mut variants, &file_name.to_lowercase());
    }

    if let Some(stripped) = lower.strip_suffix(".desktop") {
        push_variant(&mut variants, stripped);
    }

    if let Some(last_dot_segment) = lower.rsplit('.').next() {
        push_variant(&mut variants, last_dot_segment);
    }

    if let Some(prefix) = lower.strip_suffix("-gui") {
        push_variant(&mut variants, prefix);
    }

    if let Some(first_dash_segment) = lower.split('-').next() {
        if first_dash_segment != lower {
            push_variant(&mut variants, first_dash_segment);
        }
    }

    variants
}

fn push_variant(variants: &mut Vec<String>, value: &str) {
    if value.is_empty() {
        return;
    }

    let candidate = value.trim().to_string();
    if !candidate.is_empty() && !variants.contains(&candidate) {
        variants.push(candidate);
    }
}

fn walkdir(dir: &Path) -> Vec<std::io::Result<PathBuf>> {
    let mut paths = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            paths.push(Err(error));
            return paths;
        }
    };

    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    paths.extend(walkdir(&path));
                } else {
                    paths.push(Ok(path));
                }
            }
            Err(error) => paths.push(Err(error)),
        }
    }

    paths
}

fn is_yaml_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("yaml" | "yml")
    )
}

fn parse_shortcut_file(path: &Path) -> Option<ShortcutFile> {
    let content = fs::read_to_string(path).ok()?;
    let mut file: ShortcutFile = serde_yaml::from_str(&content).ok()?;
    if file.references.is_empty() {
        return None;
    }

    file.source_path = Some(path.to_path_buf());
    Some(file)
}

fn deserialize_processes<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ProcessField {
        Single(String),
        Multiple(Vec<String>),
    }

    let value = Option::<ProcessField>::deserialize(deserializer)?;
    let processes = match value {
        Some(ProcessField::Single(process)) => vec![process],
        Some(ProcessField::Multiple(processes)) => processes,
        None => Vec::new(),
    };

    Ok(processes
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect())
}

fn deserialize_string_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringField {
        Single(String),
        Multiple(Vec<String>),
    }

    let value = Option::<StringField>::deserialize(deserializer)?;
    let values = match value {
        Some(StringField::Single(value)) => vec![value],
        Some(StringField::Multiple(values)) => values,
        None => Vec::new(),
    };

    Ok(values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect())
}

fn title_pattern_matches(file: &ShortcutFile, window_title: &str) -> bool {
    let pattern_matches = match &file.title_pattern {
        Some(pattern) => Regex::new(pattern)
            .map(|regex| regex.is_match(window_title))
            .unwrap_or(false),
        None => true,
    };

    let contains_matches = match &file.title_contains {
        Some(value) => window_title.to_lowercase().contains(&value.to_lowercase()),
        None => true,
    };

    pattern_matches && contains_matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_reference_group(name: &str) -> ReferenceGroup {
        ReferenceGroup {
            group: name.to_string(),
            items: vec![ReferenceItem {
                keys: vec!["Ctrl+P".to_string()],
                label: format!("{name} item"),
                value: Some("value".to_string()),
                notes: Some("notes".to_string()),
                url: None,
                search_terms: Vec::new(),
            }],
        }
    }

    fn sample_shortcut_file(
        name: &str,
        process: &[&str],
        title_pattern: Option<&str>,
        path: &str,
    ) -> ShortcutFile {
        ShortcutFile {
            name: name.to_string(),
            process: process.iter().map(|value| value.to_string()).collect(),
            title_pattern: title_pattern.map(str::to_string),
            title_contains: None,
            references: vec![sample_reference_group(name)],
            source_path: Some(PathBuf::from(path)),
        }
    }

    fn build_map(files: &[ShortcutFile]) -> HashMap<String, ShortcutFile> {
        let mut map = HashMap::new();

        for file in files {
            if file.process.is_empty() {
                map.insert(manual_only_identity(file), file.clone());
                continue;
            }

            for process_name in &file.process {
                map.insert(process_name.to_lowercase(), file.clone());
            }
        }

        map
    }

    fn write_yaml_file(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    fn temp_dir_path(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("justpeek-{prefix}-{unique}"))
    }

    #[test]
    fn window_info_from_candidates_normalizes_process_variants() {
        let window = WindowInfo::from_candidates(
            "org.wezfurlong.wezterm",
            "WezTerm",
            ["wezterm-gui", "/usr/bin/wezterm"],
        );

        assert_eq!(
            window.process_candidates,
            vec![
                "org.wezfurlong.wezterm",
                "wezterm",
                "wezterm-gui",
                "/usr/bin/wezterm",
            ]
        );
    }

    #[test]
    fn lookup_shortcuts_prefers_direct_process_match_when_title_matches() {
        let vscode = sample_shortcut_file(
            "VS Code",
            &["code"],
            Some("workspace"),
            "/tmp/vscode.yaml",
        );
        let git = sample_shortcut_file("Git", &["gitui"], None, "/tmp/git.yaml");
        let map = build_map(&[vscode, git]);
        let window = WindowInfo::from_candidates("code", "my workspace", ["Code"]);

        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "VS Code");
        assert!(
            trace.log_lines
                .iter()
                .any(|line| line.contains("selected 'VS Code'"))
        );
    }

    #[test]
    fn lookup_shortcuts_falls_back_to_title_pattern_match() {
        let firefox = sample_shortcut_file(
            "Firefox",
            &["firefox"],
            Some("Docs"),
            "/tmp/firefox.yaml",
        );
        let slack = sample_shortcut_file("Slack", &["slack"], None, "/tmp/slack.yaml");
        let map = build_map(&[firefox, slack]);
        let window = WindowInfo::from_process_name("unknown-app", "Project Docs");

        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "Firefox");
        assert!(
            trace.log_lines
                .iter()
                .any(|line| line.contains("title pattern matched the active window"))
        );
    }

    #[test]
    fn picker_apps_are_unique_and_sorted_by_name() {
        let vscode = sample_shortcut_file(
            "VS Code",
            &["code", "Code"],
            None,
            "/tmp/vscode.yaml",
        );
        let arc = sample_shortcut_file("Arc", &["arc"], None, "/tmp/arc.yaml");
        let map = build_map(&[vscode, arc]);

        let apps = get_picker_apps(&map);

        assert_eq!(apps.len(), 2);
        assert_eq!(
            apps.iter().map(|app| app.name.as_str()).collect::<Vec<_>>(),
            vec!["Arc", "VS Code"]
        );
    }

    #[test]
    fn lookup_picker_app_returns_panel_data_for_selected_picker_id() {
        let vscode = sample_shortcut_file(
            "VS Code",
            &["code", "Code"],
            None,
            "/tmp/vscode.yaml",
        );
        let map = build_map(&[vscode.clone()]);
        let picker_id = file_identity(&vscode);

        let panel = lookup_picker_app(&map, &picker_id).unwrap();

        assert_eq!(panel.app_name, "VS Code");
        assert_eq!(panel.groups.len(), 1);
        assert_eq!(panel.groups[0].group, "VS Code");
    }

    #[test]
    fn parse_supports_process_as_single_string() {
        let dir = temp_dir_path("single-process");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "vscode.yaml",
            r#"
name: VS Code
process: code
references:
  - group: Navigation
    items:
      - label: Quick Open
"#,
        );

        let map = scan_shortcuts(&dir);
        let file = map.get("code").unwrap();

        assert_eq!(file.process, vec!["code"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_picker_only_files_without_process() {
        let dir = temp_dir_path("picker-only");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "git.yaml",
            r#"
name: Git Reference
references:
  - group: Rollback
    items:
      - label: Revert a commit
        value: git revert <commit>
"#,
        );

        let map = scan_shortcuts(&dir);
        let apps = get_picker_apps(&map);

        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Git Reference");
        assert!(apps[0].processes.is_empty());

        let window = WindowInfo::from_process_name("random-app", "anything");
        let trace = lookup_shortcuts_with_trace(&map, &window);
        assert!(trace.panel_data.is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_search_terms() {
        let dir = temp_dir_path("search-terms");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "vscode.yaml",
            r#"
name: VS Code
process: code
references:
  - group: Navigation
    items:
      - label: Command Palette
        keys: Ctrl+Shift+P
        search_terms:
          - actions
          - cmd pal
"#,
        );

        let map = scan_shortcuts(&dir);
        let file = map.get("code").unwrap();
        let item = &file.references[0].items[0];

        assert_eq!(item.search_terms, vec!["actions", "cmd pal"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_keys_as_list() {
        let dir = temp_dir_path("keys-list");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "vscode.yaml",
            r#"
name: VS Code
process: code
references:
  - group: Navigation
    items:
      - label: Quick Open
        keys:
          - Ctrl+P
          - Cmd+P
"#,
        );

        let map = scan_shortcuts(&dir);
        let file = map.get("code").unwrap();
        let item = &file.references[0].items[0];

        assert_eq!(item.keys, vec!["Ctrl+P", "Cmd+P"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_command_alias_for_value() {
        let dir = temp_dir_path("command-alias");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "git.yaml",
            r#"
name: Git Reference
references:
  - group: Rollback
    items:
      - label: Revert a commit
        command: git revert <commit>
"#,
        );

        let map = scan_shortcuts(&dir);
        let apps = get_picker_apps(&map);
        let picker_id = &apps[0].id;
        let panel = lookup_picker_app(&map, picker_id).unwrap();

        assert_eq!(panel.groups[0].items[0].value.as_deref(), Some("git revert <commit>"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_title_contains_matching() {
        let dir = temp_dir_path("title-contains");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "browser.yaml",
            r#"
name: GitHub Browser
process: firefox
title_contains: pull request
references:
  - group: Reviews
    items:
      - label: Review changes
"#,
        );

        let map = scan_shortcuts(&dir);
        let window = WindowInfo::from_process_name("firefox", "My Pull Request - GitHub");
        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "GitHub Browser");

        let miss = WindowInfo::from_process_name("firefox", "Homepage");
        let miss_trace = lookup_shortcuts_with_trace(&map, &miss);
        assert!(miss_trace.panel_data.is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_item_url() {
        let dir = temp_dir_path("item-url");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "git.yaml",
            r#"
name: Git Reference
references:
  - group: Rollback
    items:
      - label: Revert a commit
        command: git revert <commit>
        url: https://git-scm.com/docs/git-revert
"#,
        );

        let map = scan_shortcuts(&dir);
        let apps = get_picker_apps(&map);
        let panel = lookup_picker_app(&map, &apps[0].id).unwrap();

        assert_eq!(
            panel.groups[0].items[0].url.as_deref(),
            Some("https://git-scm.com/docs/git-revert")
        );

        fs::remove_dir_all(&dir).unwrap();
    }
}
