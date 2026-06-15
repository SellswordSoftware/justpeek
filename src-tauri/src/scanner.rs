use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutFile {
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
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
    #[serde(default, deserialize_with = "deserialize_os_string_lists")]
    pub keys_by_os: BTreeMap<String, Vec<String>>,
    pub label: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
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
    pub group: String,
    pub processes: Vec<String>,
    pub source_path: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LookupTrace {
    pub panel_data: Option<PanelData>,
    pub log_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub shortcut_map: HashMap<String, ShortcutFile>,
    pub picker_apps: Vec<PickerApp>,
    pub picker_panels: HashMap<String, PanelData>,
}

pub fn scan_shortcuts(dir: &Path) -> ScanResult {
    let mut shortcut_map = HashMap::new();
    let mut picker_apps = Vec::new();
    let mut picker_panels = HashMap::new();

    if !dir.exists() {
        fs::create_dir_all(dir).expect("Failed to create references directory");
        return ScanResult {
            shortcut_map,
            picker_apps,
            picker_panels,
        };
    }

    for entry in walkdir(dir) {
        let Ok(path) = entry else {
            continue;
        };

        if !is_yaml_file(&path) {
            continue;
        }

        match parse_shortcut_file(&path) {
            Ok(file) => {
                let picker_app = picker_app_for_valid_file(&file);
                picker_panels.insert(picker_app.id.clone(), panel_data_for(&file));
                picker_apps.push(picker_app);

                if file.process.is_empty() {
                    shortcut_map.insert(manual_only_identity(&file), file);
                    continue;
                }

                for process_name in &file.process {
                    shortcut_map.insert(process_name.to_lowercase(), file.clone());
                }
            }
            Err(error) => {
                let picker_app = picker_app_for_invalid_file(&path, &error);
                picker_panels.insert(
                    picker_app.id.clone(),
                    error_panel_data_for_path(&path, &picker_app.name, &error),
                );
                picker_apps.push(picker_app);
            }
        }
    }

    picker_apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));

    ScanResult {
        shortcut_map,
        picker_apps,
        picker_panels,
    }
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
    let mut direct_unconstrained_match: Option<&ShortcutFile> = None;
    for candidate in &window.process_candidates {
        if let Some(file) = map.get(candidate) {
            direct_match_found = true;
            let has_title_constraints =
                file.title_pattern.is_some() || file.title_contains.is_some();
            let title_match = title_pattern_matches(file, &window.window_title);
            log_lines.push(format!(
                "direct process match: '{}' -> {} [{}]",
                candidate,
                describe_file(file),
                describe_title_check(file, title_match),
            ));

            if has_title_constraints && title_match {
                log_lines.push(format!(
                    "selected {} because process matched '{}' and title-constrained check passed",
                    describe_file(file),
                    candidate,
                ));

                return LookupTrace {
                    panel_data: Some(panel_data_for(file)),
                    log_lines,
                };
            }

            if !has_title_constraints && direct_unconstrained_match.is_none() {
                direct_unconstrained_match = Some(file);
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
        let has_title_constraints = file.title_pattern.is_some() || file.title_contains.is_some();
        let title_match = title_pattern_matches(file, &window.window_title);
        log_lines.push(format!(
            "fallback candidate: {} [{}]",
            describe_file(file),
            describe_title_check(file, title_match),
        ));

        if has_title_constraints && title_match {
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

    if let Some(file) = direct_unconstrained_match {
        log_lines.push(format!(
            "selected {} because it matched the active process and no title-constrained reference matched",
            describe_file(file),
        ));

        return LookupTrace {
            panel_data: Some(panel_data_for(file)),
            log_lines,
        };
    }

    log_lines.push("no reference file matched the active window".to_string());

    LookupTrace {
        panel_data: None,
        log_lines,
    }
}

#[cfg(test)]
pub fn get_picker_apps(scan_result: &ScanResult) -> Vec<PickerApp> {
    let mut apps = scan_result.picker_apps.clone();
    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    apps
}

#[cfg(test)]
pub fn lookup_picker_app(scan_result: &ScanResult, picker_id: &str) -> Option<PanelData> {
    scan_result.picker_panels.get(picker_id).cloned()
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

fn picker_app_for_valid_file(file: &ShortcutFile) -> PickerApp {
    let source_path = file
        .source_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<unknown path>".to_string());

    PickerApp {
        id: file_identity(file),
        name: file.name.clone(),
        group: picker_group_name(file.group.as_deref()),
        processes: file.process.clone(),
        source_path,
        error: None,
    }
}

fn picker_app_for_invalid_file(path: &Path, error: &str) -> PickerApp {
    PickerApp {
        id: path.to_string_lossy().into_owned(),
        name: fallback_reference_name(path),
        group: "Invalid".to_string(),
        processes: Vec::new(),
        source_path: path.to_string_lossy().into_owned(),
        error: Some(error.to_string()),
    }
}

fn panel_data_for(file: &ShortcutFile) -> PanelData {
    PanelData {
        app_name: file.name.clone(),
        groups: file.references.clone(),
    }
}

fn error_panel_data_for_path(path: &Path, name: &str, error: &str) -> PanelData {
    PanelData {
        app_name: name.to_string(),
        groups: vec![ReferenceGroup {
            group: "Reference File Error".to_string(),
            items: vec![
                ReferenceItem {
                    keys: Vec::new(),
                    keys_by_os: BTreeMap::new(),
                    label: "Issue".to_string(),
                    value: Some(error.to_string()),
                    command: None,
                    notes: Some("Fix the file and reload references.".to_string()),
                    url: None,
                    search_terms: Vec::new(),
                },
                ReferenceItem {
                    keys: Vec::new(),
                    keys_by_os: BTreeMap::new(),
                    label: "File".to_string(),
                    value: Some(path.to_string_lossy().into_owned()),
                    command: None,
                    notes: None,
                    url: None,
                    search_terms: Vec::new(),
                },
            ],
        }],
    }
}

fn fallback_reference_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Invalid Reference File")
        .to_string()
}

fn picker_group_name(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Ungrouped")
        .to_string()
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
    pub fn from_process_name(
        process_name: impl Into<String>,
        window_title: impl Into<String>,
    ) -> Self {
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
            process_candidates: normalized_process_candidates(
                candidate_inputs.iter().map(String::as_str),
            ),
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

fn parse_shortcut_file(path: &Path) -> Result<ShortcutFile, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read file: {error}"))?;
    let mut file: ShortcutFile = serde_yaml::from_str(&content)
        .map_err(|error| format!("YAML parse error: {error}"))?;
    if file.references.is_empty() {
        return Err("Reference file must include at least one group under 'references'.".to_string());
    }

    file.source_path = Some(path.to_path_buf());
    Ok(file)
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

fn deserialize_os_string_lists<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<BTreeMap<String, serde_yaml::Value>>::deserialize(deserializer)?;
    let Some(entries) = value else {
        return Ok(BTreeMap::new());
    };

    let mut parsed = BTreeMap::new();
    for (raw_key, raw_value) in entries {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringField {
            Single(String),
            Multiple(Vec<String>),
        }

        let key = raw_key.trim().to_ascii_lowercase();
        if !matches!(key.as_str(), "macos" | "windows" | "linux") {
            return Err(serde::de::Error::custom(format!(
                "Unsupported keys_by_os platform '{raw_key}'. Use one of: macos, windows, linux."
            )));
        }

        let values = serde_yaml::from_value::<StringField>(raw_value)
            .map_err(|error| serde::de::Error::custom(error.to_string()))?;
        let values = match values {
            StringField::Single(value) => vec![value],
            StringField::Multiple(values) => values,
        }
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();

        parsed.insert(key, values);
    }

    Ok(parsed)
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
                keys_by_os: BTreeMap::new(),
                label: format!("{name} item"),
                value: Some("value".to_string()),
                command: None,
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
            group: None,
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
        let vscode =
            sample_shortcut_file("VS Code", &["code"], Some("workspace"), "/tmp/vscode.yaml");
        let git = sample_shortcut_file("Git", &["gitui"], None, "/tmp/git.yaml");
        let map = build_map(&[vscode, git]);
        let window = WindowInfo::from_candidates("code", "my workspace", ["Code"]);

        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "VS Code");
        assert!(trace
            .log_lines
            .iter()
            .any(|line| line.contains("selected 'VS Code'")));
    }

    #[test]
    fn lookup_shortcuts_falls_back_to_title_pattern_match() {
        let firefox =
            sample_shortcut_file("Firefox", &["firefox"], Some("Docs"), "/tmp/firefox.yaml");
        let slack = sample_shortcut_file("Slack", &["slack"], None, "/tmp/slack.yaml");
        let map = build_map(&[firefox, slack]);
        let window = WindowInfo::from_process_name("unknown-app", "Project Docs");

        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "Firefox");
        assert!(trace
            .log_lines
            .iter()
            .any(|line| line.contains("title pattern matched the active window")));
    }

    #[test]
    fn lookup_shortcuts_prefers_title_match_over_generic_process_match() {
        let chrome = sample_shortcut_file("Chrome", &["chrome"], None, "/tmp/chrome.yaml");
        let mut github =
            sample_shortcut_file("GitHub", &["firefox"], None, "/tmp/github.yaml");
        github.title_contains = Some("github".to_string());
        let map = build_map(&[chrome, github]);
        let window = WindowInfo::from_candidates("chrome", "OpenAI - GitHub", ["google-chrome"]);

        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "GitHub");
        assert!(trace
            .log_lines
            .iter()
            .any(|line| line.contains("selected 'GitHub'")));
    }

    #[test]
    fn lookup_shortcuts_does_not_fallback_to_unconstrained_process_file() {
        let git = sample_shortcut_file("Git Reference", &["gitui"], None, "/tmp/git.yaml");
        let map = build_map(&[git]);
        let window = WindowInfo::from_candidates(
            "vivaldi-stable",
            "window | Tauri - Vivaldi",
            ["vivaldi", "vivaldi-bin"],
        );

        let trace = lookup_shortcuts_with_trace(&map, &window);

        assert!(trace.panel_data.is_none());
        assert!(trace
            .log_lines
            .iter()
            .any(|line| line.contains("fallback candidate: 'Git Reference'")));
        assert!(trace
            .log_lines
            .iter()
            .any(|line| line.contains("matched=true")));
        assert!(trace
            .log_lines
            .iter()
            .any(|line| line.contains("no reference file matched the active window")));
    }

    #[test]
    fn picker_apps_are_unique_and_sorted_by_name() {
        let vscode = sample_shortcut_file("VS Code", &["code", "Code"], None, "/tmp/vscode.yaml");
        let arc = sample_shortcut_file("Arc", &["arc"], None, "/tmp/arc.yaml");
        let scan_result = ScanResult {
            shortcut_map: build_map(&[vscode, arc]),
            picker_apps: vec![
                PickerApp {
                    id: "/tmp/vscode.yaml".to_string(),
                    name: "VS Code".to_string(),
                    group: "Editors".to_string(),
                    processes: vec!["code".to_string(), "Code".to_string()],
                    source_path: "/tmp/vscode.yaml".to_string(),
                    error: None,
                },
                PickerApp {
                    id: "/tmp/arc.yaml".to_string(),
                    name: "Arc".to_string(),
                    group: "Browsers".to_string(),
                    processes: vec!["arc".to_string()],
                    source_path: "/tmp/arc.yaml".to_string(),
                    error: None,
                },
            ],
            picker_panels: HashMap::new(),
        };

        let apps = get_picker_apps(&scan_result);

        assert_eq!(apps.len(), 2);
        assert_eq!(
            apps.iter().map(|app| app.name.as_str()).collect::<Vec<_>>(),
            vec!["Arc", "VS Code"]
        );
    }

    #[test]
    fn lookup_picker_app_returns_panel_data_for_selected_picker_id() {
        let vscode = sample_shortcut_file("VS Code", &["code", "Code"], None, "/tmp/vscode.yaml");
        let panel = panel_data_for(&vscode);
        let picker_id = file_identity(&vscode);
        let scan_result = ScanResult {
            shortcut_map: build_map(&[vscode]),
            picker_apps: Vec::new(),
            picker_panels: HashMap::from([(picker_id.clone(), panel)]),
        };

        let panel = lookup_picker_app(&scan_result, &picker_id).unwrap();

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

        let scan_result = scan_shortcuts(&dir);
        let file = scan_result.shortcut_map.get("code").unwrap();

        assert_eq!(file.process, vec!["code"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_top_level_picker_group() {
        let dir = temp_dir_path("picker-group");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "vscode.yaml",
            r#"
name: VS Code
group: Editors
process: code
references:
  - group: Navigation
    items:
      - label: Quick Open
"#,
        );

        let scan_result = scan_shortcuts(&dir);
        let apps = get_picker_apps(&scan_result);

        assert_eq!(scan_result.shortcut_map.get("code").unwrap().group.as_deref(), Some("Editors"));
        assert_eq!(apps[0].group, "Editors");

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

        let scan_result = scan_shortcuts(&dir);
        let apps = get_picker_apps(&scan_result);

        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Git Reference");
        assert_eq!(apps[0].group, "Ungrouped");
        assert!(apps[0].processes.is_empty());
        assert_eq!(apps[0].error, None);

        let window = WindowInfo::from_process_name("random-app", "anything");
        let trace = lookup_shortcuts_with_trace(&scan_result.shortcut_map, &window);
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

        let scan_result = scan_shortcuts(&dir);
        let file = scan_result.shortcut_map.get("code").unwrap();
        let item = &file.references[0].items[0];

        assert_eq!(item.search_terms, vec!["actions", "cmd pal"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_keeps_value_and_command_distinct() {
        let dir = temp_dir_path("value-and-command");
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
        value: Safe for shared history
        command: git revert <commit>
"#,
        );

        let scan_result = scan_shortcuts(&dir);
        let apps = get_picker_apps(&scan_result);
        let panel = lookup_picker_app(&scan_result, &apps[0].id).unwrap();
        let item = &panel.groups[0].items[0];

        assert_eq!(item.value.as_deref(), Some("Safe for shared history"));
        assert_eq!(item.command.as_deref(), Some("git revert <commit>"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_keys_by_os() {
        let dir = temp_dir_path("keys-by-os");
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
        keys: Ctrl+P
        keys_by_os:
          macos: Cmd+P
          windows:
            - Ctrl+P
            - Ctrl+Shift+P
"#,
        );

        let scan_result = scan_shortcuts(&dir);
        let file = scan_result.shortcut_map.get("code").unwrap();
        let item = &file.references[0].items[0];

        assert_eq!(item.keys, vec!["Ctrl+P"]);
        assert_eq!(
            item.keys_by_os.get("macos").cloned(),
            Some(vec!["Cmd+P".to_string()])
        );
        assert_eq!(
            item.keys_by_os.get("windows").cloned(),
            Some(vec!["Ctrl+P".to_string(), "Ctrl+Shift+P".to_string()])
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_rejects_unknown_keys_by_os_platform() {
        let content = r#"
name: VS Code
process: code
references:
  - group: Navigation
    items:
      - label: Quick Open
        keys_by_os:
          bsd: Ctrl+P
"#;

        let result = serde_yaml::from_str::<ShortcutFile>(content);

        assert!(result.is_err());
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

        let scan_result = scan_shortcuts(&dir);
        let file = scan_result.shortcut_map.get("code").unwrap();
        let item = &file.references[0].items[0];

        assert_eq!(item.keys, vec!["Ctrl+P", "Cmd+P"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_supports_command_field() {
        let dir = temp_dir_path("command-field");
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

        let scan_result = scan_shortcuts(&dir);
        let apps = get_picker_apps(&scan_result);
        let picker_id = &apps[0].id;
        let panel = lookup_picker_app(&scan_result, picker_id).unwrap();

        assert_eq!(
            panel.groups[0].items[0].command.as_deref(),
            Some("git revert <commit>")
        );
        assert_eq!(panel.groups[0].items[0].value.as_deref(), None);

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

        let scan_result = scan_shortcuts(&dir);
        let window = WindowInfo::from_process_name("firefox", "My Pull Request - GitHub");
        let trace = lookup_shortcuts_with_trace(&scan_result.shortcut_map, &window);

        assert_eq!(trace.panel_data.unwrap().app_name, "GitHub Browser");

        let miss = WindowInfo::from_process_name("firefox", "Homepage");
        let miss_trace = lookup_shortcuts_with_trace(&scan_result.shortcut_map, &miss);
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

        let scan_result = scan_shortcuts(&dir);
        let apps = get_picker_apps(&scan_result);
        let panel = lookup_picker_app(&scan_result, &apps[0].id).unwrap();

        assert_eq!(
            panel.groups[0].items[0].url.as_deref(),
            Some("https://git-scm.com/docs/git-revert")
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn invalid_reference_files_still_appear_in_picker_with_error_panel() {
        let dir = temp_dir_path("invalid-picker");
        fs::create_dir_all(&dir).unwrap();
        write_yaml_file(
            &dir,
            "broken.yaml",
            r#"
name: Broken Reference
references:
  - group: Broken
    items:
      - label Oops
"#,
        );

        let scan_result = scan_shortcuts(&dir);
        let apps = get_picker_apps(&scan_result);

        assert_eq!(scan_result.shortcut_map.len(), 0);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "broken");
        assert_eq!(apps[0].group, "Invalid");
        assert!(apps[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("YAML parse error")));

        let panel = lookup_picker_app(&scan_result, &apps[0].id).unwrap();
        assert_eq!(panel.app_name, "broken");
        assert_eq!(panel.groups[0].group, "Reference File Error");
        assert_eq!(panel.groups[0].items[0].label, "Issue");

        fs::remove_dir_all(&dir).unwrap();
    }
}
