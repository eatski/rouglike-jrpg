/// Cargo.toml からワークスペース内crate間の依存関係をMermaid図として出力する
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn main() {
    let root = find_workspace_root();
    let workspace_toml = root.join("Cargo.toml");
    let content = fs::read_to_string(&workspace_toml).expect("Failed to read workspace Cargo.toml");

    let members = parse_workspace_members(&content);
    let root_name = parse_package_name(&content).unwrap_or_default();

    // 全crateの名前を収集
    let mut known_crates: BTreeSet<String> = BTreeSet::new();
    let mut member_tomls: Vec<(String, PathBuf)> = Vec::new();

    for member_dir in &members {
        // tools/ 配下のcrateは図に含めない
        if member_dir.starts_with("tools/") {
            continue;
        }
        let toml_path = root.join(member_dir).join("Cargo.toml");
        if let Ok(toml_content) = fs::read_to_string(&toml_path) {
            if let Some(name) = parse_package_name(&toml_content) {
                known_crates.insert(name.clone());
                member_tomls.push((name, toml_path));
            }
        }
    }
    known_crates.insert(root_name.clone());

    // 依存関係を収集
    let mut deps: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // ルートパッケージの依存
    let root_deps = parse_path_dependencies(&content, &known_crates);
    if !root_deps.is_empty() {
        deps.insert(root_name.clone(), root_deps);
    }

    // 各メンバーcrateの依存
    for (name, toml_path) in &member_tomls {
        if let Ok(toml_content) = fs::read_to_string(toml_path) {
            let crate_deps = parse_path_dependencies(&toml_content, &known_crates);
            if !crate_deps.is_empty() {
                deps.insert(name.clone(), crate_deps);
            }
        }
    }

    // Mermaid出力
    print_mermaid(&root_name, &known_crates, &deps);
}

fn find_workspace_root() -> PathBuf {
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let mut dir = cwd.as_path();
    loop {
        let toml = dir.join("Cargo.toml");
        if toml.exists() {
            let content = fs::read_to_string(&toml).unwrap_or_default();
            if content.contains("[workspace]") {
                return dir.to_path_buf();
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                eprintln!("Error: Could not find workspace root. Run from within the project.");
                std::process::exit(1);
            }
        }
    }
}

fn parse_package_name(content: &str) -> Option<String> {
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
            continue;
        }
        if trimmed.starts_with('[') {
            if in_package {
                break;
            }
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            return extract_quoted_value(trimmed);
        }
    }
    None
}

fn parse_workspace_members(content: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut in_members = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed == "]" {
                break;
            }
            if let Some(val) = extract_quoted_value(trimmed) {
                members.push(val);
            }
        }
    }
    members
}

fn parse_path_dependencies(content: &str, known: &BTreeSet<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_deps = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]" {
            in_deps = true;
            continue;
        }
        if trimmed.starts_with('[') {
            if in_deps {
                break;
            }
            continue;
        }
        if in_deps && trimmed.contains("path") {
            if let Some(dep_name) = trimmed.split('=').next() {
                let dep_name = dep_name.trim().to_string();
                if known.contains(&dep_name) {
                    result.push(dep_name);
                }
            }
        }
    }
    result
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

fn mermaid_id(name: &str) -> String {
    name.replace('-', "_")
}

#[derive(PartialEq)]
enum Layer {
    Domain,
    UiCommon,
    UiFeature,
    App,
}

fn classify(name: &str) -> Layer {
    match name {
        "terrain" | "party" | "battle" | "cave" | "town" | "world" | "time" => Layer::Domain,
        "app-state" | "movement-ui" | "input-ui" | "shared-ui" => Layer::UiCommon,
        "world-ui" | "cave-ui" | "town-ui" | "battle-ui" | "time-ui" => Layer::UiFeature,
        _ => Layer::App,
    }
}

fn print_mermaid(
    root_name: &str,
    known_crates: &BTreeSet<String>,
    deps: &BTreeMap<String, Vec<String>>,
) {
    println!("```mermaid");
    println!("graph TD");
    println!();

    println!("    classDef domain fill:#a8d5ba,stroke:#2d6a4f,color:#1b4332");
    println!("    classDef uiCommon fill:#a2d2ff,stroke:#0077b6,color:#023e8a");
    println!("    classDef uiFeature fill:#ffb3c6,stroke:#c9184a,color:#590d22");
    println!("    classDef app fill:#e0aaff,stroke:#7b2cbf,color:#3c096c");
    println!();

    print_subgraph("domain", "Domain Layer", known_crates, |n| {
        classify(n) == Layer::Domain
    });

    print_subgraph("ui_common", "UI Common Layer", known_crates, |n| {
        classify(n) == Layer::UiCommon
    });

    print_subgraph("ui_feature", "UI Feature Layer", known_crates, |n| {
        classify(n) == Layer::UiFeature
    });

    println!("    {}[\"{}\"]", mermaid_id(root_name), root_name);
    println!();

    for (from, to_list) in deps {
        for to in to_list {
            println!("    {} --> {}", mermaid_id(from), mermaid_id(to));
        }
    }
    println!();

    for name in known_crates {
        let class = match classify(name) {
            Layer::Domain => "domain",
            Layer::UiCommon => "uiCommon",
            Layer::UiFeature => "uiFeature",
            Layer::App => "app",
        };
        println!("    class {} {}", mermaid_id(name), class);
    }

    println!("```");
}

fn print_subgraph(
    id: &str,
    label: &str,
    known_crates: &BTreeSet<String>,
    filter: impl Fn(&str) -> bool,
) {
    let members: Vec<&String> = known_crates.iter().filter(|n| filter(n)).collect();
    if members.is_empty() {
        return;
    }
    println!("    subgraph {}[\"{}\"]", id, label);
    for name in members {
        println!("        {}[\"{}\"]", mermaid_id(name), name);
    }
    println!("    end");
    println!();
}
