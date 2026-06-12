use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutFile {
    pub name: String,
    pub process: Vec<String>,
    #[serde(default)]
    pub title_pattern: Option<String>,
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
    #[serde(default)]
    pub keys: Option<String>,
    pub label: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
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

    for file in unique_files(map) {
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
    match &file.title_pattern {
        Some(pattern) => format!("title_pattern={pattern:?} matched={matched}"),
        None => "title_pattern=<none> matched=true".to_string(),
    }
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
    if file.process.is_empty() || file.references.is_empty() {
        return None;
    }

    file.source_path = Some(path.to_path_buf());
    Some(file)
}

fn title_pattern_matches(file: &ShortcutFile, window_title: &str) -> bool {
    match &file.title_pattern {
        Some(pattern) => Regex::new(pattern)
            .map(|regex| regex.is_match(window_title))
            .unwrap_or(false),
        None => true,
    }
}
