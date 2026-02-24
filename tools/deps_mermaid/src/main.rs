/// Cargo.toml からワークスペース内crate間の依存関係をMermaid図として出力する
/// クラスタリングはモジュラリティ最適化（Louvain風）でクラスタ間結合度を最小化
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn main() {
    let root = find_workspace_root();
    let workspace_toml = root.join("Cargo.toml");
    let content = fs::read_to_string(&workspace_toml).expect("Failed to read workspace Cargo.toml");

    let members = parse_workspace_members(&content);
    let root_name = parse_package_name(&content).unwrap_or_default();

    // 全crateの名前を収集（crates/配下のみ）
    let mut known_crates: BTreeSet<String> = BTreeSet::new();
    let mut member_tomls: Vec<(String, PathBuf)> = Vec::new();

    for member_dir in &members {
        // crates/ 配下のみ含める
        if !member_dir.starts_with("crates/") {
            continue;
        }
        let toml_path = root.join(member_dir).join("Cargo.toml");
        if let Ok(toml_content) = fs::read_to_string(&toml_path)
            && let Some(name) = parse_package_name(&toml_content)
        {
            known_crates.insert(name.clone());
            member_tomls.push((name, toml_path));
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
    print_mermaid(&known_crates, &deps);
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
        if in_deps && trimmed.contains("path")
            && let Some(dep_name) = trimmed.split('=').next()
        {
            let dep_name = dep_name.trim().to_string();
            if known.contains(&dep_name) {
                result.push(dep_name);
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

// ─── モジュラリティベースのクラスタリング ─────────────────────────

/// グラフのモジュラリティを計算（Q値）
/// Q = (1/2m) Σ [A_ij - k_i*k_j/(2m)] δ(c_i, c_j)
fn modularity(community: &[usize], adj: &[Vec<bool>], degree: &[usize], m: f64) -> f64 {
    let n = community.len();
    let mut q = 0.0;
    for i in 0..n {
        for j in 0..n {
            if community[i] == community[j] {
                let a_ij = if adj[i][j] { 1.0 } else { 0.0 };
                q += a_ij - (degree[i] as f64 * degree[j] as f64) / (2.0 * m);
            }
        }
    }
    q / (2.0 * m)
}

/// 貪欲モジュラリティ最適化でクラスタを算出（Louvain phase 1）
/// 返り値: (クラスタ名, メンバー一覧) のリスト（平均深度の浅い順）
fn compute_clusters(
    known_crates: &BTreeSet<String>,
    deps: &BTreeMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    let nodes: Vec<String> = known_crates.iter().cloned().collect();
    let n = nodes.len();
    if n == 0 {
        return Vec::new();
    }

    let node_idx: BTreeMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), i))
        .collect();

    // 無向隣接行列を構築
    let mut adj = vec![vec![false; n]; n];
    let mut total_edges = 0usize;
    for (from, to_list) in deps {
        if let Some(&fi) = node_idx.get(from.as_str()) {
            for to in to_list {
                if let Some(&ti) = node_idx.get(to.as_str()) {
                    if !adj[fi][ti] {
                        adj[fi][ti] = true;
                        adj[ti][fi] = true;
                        total_edges += 1;
                    }
                }
            }
        }
    }

    if total_edges == 0 {
        return vec![nodes];
    }

    let m = total_edges as f64;
    let degree: Vec<usize> = (0..n)
        .map(|i| (0..n).filter(|&j| adj[i][j]).count())
        .collect();

    // 各ノードを個別のコミュニティとして開始
    let mut community: Vec<usize> = (0..n).collect();

    // 貪欲最適化: 各ノードをモジュラリティが最大化する隣接コミュニティへ移動
    let mut improved = true;
    while improved {
        improved = false;
        for i in 0..n {
            let current_comm = community[i];
            let mut best_comm = current_comm;
            let mut best_q = modularity(&community, &adj, &degree, m);

            // 隣接ノードのコミュニティを候補として試す
            let neighbor_comms: BTreeSet<usize> = (0..n)
                .filter(|&j| adj[i][j] && community[j] != current_comm)
                .map(|j| community[j])
                .collect();

            for &target in &neighbor_comms {
                community[i] = target;
                let q = modularity(&community, &adj, &degree, m);
                if q > best_q {
                    best_q = q;
                    best_comm = target;
                }
            }

            if best_comm != current_comm {
                community[i] = best_comm;
                improved = true;
            } else {
                community[i] = current_comm;
            }
        }
    }

    // コミュニティIDでグループ化
    let mut groups: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (i, &comm) in community.iter().enumerate() {
        groups.entry(comm).or_default().push(nodes[i].clone());
    }

    // 深度情報で各クラスタをソート（基盤的なクラスタが先）
    let depths = compute_depths(known_crates, deps);
    let mut clusters: Vec<Vec<String>> = groups.into_values().collect();
    clusters.sort_by(|a, b| {
        let avg_a: f64 =
            a.iter().map(|n| depths.get(n).copied().unwrap_or(0) as f64).sum::<f64>() / a.len() as f64;
        let avg_b: f64 =
            b.iter().map(|n| depths.get(n).copied().unwrap_or(0) as f64).sum::<f64>() / b.len() as f64;
        avg_a.partial_cmp(&avg_b).unwrap()
    });

    clusters
}

// ─── 深度計算（色付け用） ──────────────────────────────────────

/// 各crateのトポロジカル深度を算出（依存がないcrate=0、依存先の最大深度+1）
fn compute_depths(
    known_crates: &BTreeSet<String>,
    deps: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, usize> {
    let mut memo: BTreeMap<String, usize> = BTreeMap::new();
    for name in known_crates {
        depth_of(name, deps, &mut memo);
    }
    memo
}

fn depth_of(
    name: &str,
    deps: &BTreeMap<String, Vec<String>>,
    memo: &mut BTreeMap<String, usize>,
) -> usize {
    if let Some(&d) = memo.get(name) {
        return d;
    }
    let d = match deps.get(name) {
        Some(dep_list) if !dep_list.is_empty() => {
            dep_list
                .iter()
                .map(|d| depth_of(d, deps, memo))
                .max()
                .unwrap()
                + 1
        }
        _ => 0,
    };
    memo.insert(name.to_string(), d);
    d
}

// ─── Mermaid出力 ──────────────────────────────────────────────

/// クラスタごとの色スタイル定義
const CLUSTER_STYLES: &[&str] = &[
    "fill:#a8d5ba,stroke:#2d6a4f,color:#1b4332", // 緑
    "fill:#a2d2ff,stroke:#0077b6,color:#023e8a", // 青
    "fill:#ffb3c6,stroke:#c9184a,color:#590d22", // ピンク
    "fill:#e0aaff,stroke:#7b2cbf,color:#3c096c", // 紫
    "fill:#ffd6a5,stroke:#e85d04,color:#6a040f", // オレンジ
    "fill:#fdffb6,stroke:#b5a100,color:#414833", // 黄
];

fn print_mermaid(known_crates: &BTreeSet<String>, deps: &BTreeMap<String, Vec<String>>) {
    let clusters = compute_clusters(known_crates, deps);

    println!("```mermaid");
    println!("graph TD");
    println!();

    // classDef定義（クラスタごと）
    for (i, _) in clusters.iter().enumerate() {
        let style = CLUSTER_STYLES[i % CLUSTER_STYLES.len()];
        println!("    classDef cluster{} {}", i, style);
    }
    println!();

    // subgraph出力（クラスタ単位）
    for (i, members) in clusters.iter().enumerate() {
        let label = members.join(", ");
        println!("    subgraph cluster_{}[\"{}\"]", i, label);
        for name in members {
            println!("        {}[\"{}\"]", mermaid_id(name), name);
        }
        println!("    end");
        println!();
    }

    // エッジ
    for (from, to_list) in deps {
        if !known_crates.contains(from) {
            continue;
        }
        for to in to_list {
            println!("    {} --> {}", mermaid_id(from), mermaid_id(to));
        }
    }
    println!();

    // class適用（クラスタごとの色）
    for (i, members) in clusters.iter().enumerate() {
        for name in members {
            println!("    class {} cluster{}", mermaid_id(name), i);
        }
    }

    println!("```");
}
