use std::fmt;
use std::process::{Command, exit};

use console::Style;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;

struct WorktreeInfo {
    branch: String,
    last_commit_message: String,
    last_commit_relative_date: String,
    commits_ahead: u32,
}

impl fmt::Display for WorktreeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dim = Style::new().dim();
        let commits_label = if self.commits_ahead == 1 {
            "commit "
        } else {
            "commits"
        };
        write!(
            f,
            "{}\n  {} {} {} {} {} {} {}",
            Style::new().bold().apply_to(&self.branch),
            dim.apply_to("│"),
            self.commits_ahead,
            dim.apply_to(commits_label),
            dim.apply_to("│"),
            dim.apply_to(&self.last_commit_relative_date),
            dim.apply_to("│"),
            dim.apply_to(&self.last_commit_message),
        )
    }
}

fn repo_root() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("git rev-parse failed");
    String::from_utf8(output.stdout)
        .expect("invalid utf8")
        .trim()
        .to_string()
}

fn get_worktree_branches(repo: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["-C", repo, "worktree", "list", "--porcelain"])
        .output()
        .expect("git worktree list failed");
    let text = String::from_utf8(output.stdout).expect("invalid utf8");

    let mut branches = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("branch refs/heads/") {
            if rest != "main" {
                branches.push(rest.to_string());
            }
        }
    }
    branches
}

fn get_worktree_info(repo: &str, branch: &str) -> WorktreeInfo {
    // 最新コミットメッセージと相対日時
    let log_output = Command::new("git")
        .args([
            "-C",
            repo,
            "log",
            "-1",
            "--format=%s\n%ar",
            branch,
        ])
        .output()
        .expect("git log failed");
    let log_text = String::from_utf8(log_output.stdout).expect("invalid utf8");
    let mut lines = log_text.trim().lines();
    let last_commit_message = lines.next().unwrap_or("").to_string();
    let last_commit_relative_date = lines.next().unwrap_or("").to_string();

    // mainからのコミット数
    let rev_list_output = Command::new("git")
        .args([
            "-C",
            repo,
            "rev-list",
            "--count",
            &format!("main..{branch}"),
        ])
        .output()
        .expect("git rev-list failed");
    let count_text = String::from_utf8(rev_list_output.stdout).expect("invalid utf8");
    let commits_ahead = count_text.trim().parse::<u32>().unwrap_or(0);

    WorktreeInfo {
        branch: branch.to_string(),
        last_commit_message,
        last_commit_relative_date,
        commits_ahead,
    }
}

fn main() {
    let repo = repo_root();
    let branches = get_worktree_branches(&repo);

    if branches.is_empty() {
        eprintln!("作業中のworktreeブランチがありません");
        exit(1);
    }

    let items: Vec<WorktreeInfo> = branches
        .iter()
        .map(|b| get_worktree_info(&repo, b))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("マージするブランチを選択してください")
        .items(&items)
        .default(0)
        .interact_opt()
        .expect("選択UIでエラーが発生しました");

    let selected = match selection {
        Some(i) => &items[i].branch,
        None => {
            eprintln!("キャンセルされました");
            exit(0);
        }
    };

    println!();
    println!("ブランチ '{}' をmainにマージします...", selected);

    // mainブランチに切り替え
    let current = Command::new("git")
        .args(["-C", &repo, "branch", "--show-current"])
        .output()
        .expect("git branch failed");
    let current_branch = String::from_utf8(current.stdout)
        .expect("invalid utf8")
        .trim()
        .to_string();

    if current_branch != "main" {
        println!("mainブランチに切り替えます...");
        let status = Command::new("git")
            .args(["-C", &repo, "checkout", "main"])
            .status()
            .expect("git checkout failed");
        if !status.success() {
            eprintln!("mainブランチへの切り替えに失敗しました");
            exit(1);
        }
    }

    // マージ実行
    let merge_status = Command::new("git")
        .args(["-C", &repo, "merge", "--no-edit", selected])
        .status()
        .expect("git merge failed");

    if merge_status.success() {
        println!();
        println!("マージ成功: {} -> main", selected);
    } else {
        println!();
        println!("コンフリクトが発生しました。Claude Codeでコンフリクト解消を開始します...");
        println!();

        // コンフリクトファイル一覧
        let diff_output = Command::new("git")
            .args(["-C", &repo, "diff", "--name-only", "--diff-filter=U"])
            .output()
            .expect("git diff failed");
        let conflict_files = String::from_utf8(diff_output.stdout)
            .expect("invalid utf8")
            .trim()
            .to_string();

        println!("コンフリクトファイル:");
        for file in conflict_files.lines() {
            println!("  {}", file);
        }
        println!();

        // Claude Codeを起動
        let prompt = format!(
            "git mergeでコンフリクトが発生しました。以下のファイルのコンフリクトを解消してください。\
            コンフリクトマーカー（<<<<<<< ======= >>>>>>>）を確認し、両方の変更を適切に統合してください。\
            解消後、git addして git commit --no-edit してください。\n\n\
            コンフリクトファイル:\n{}",
            conflict_files
        );

        let claude_status = Command::new("claude")
            .arg(&prompt)
            .current_dir(&repo)
            .status();

        match claude_status {
            Ok(s) if !s.success() => {
                eprintln!("Claude Codeがエラーで終了しました");
                exit(1);
            }
            Err(e) => {
                eprintln!("Claude Codeの起動に失敗しました: {}", e);
                exit(1);
            }
            _ => {}
        }
    }
}
