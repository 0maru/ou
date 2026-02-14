---
globs: src/**/*.rs, tests/**/*.rs
---

# アーキテクチャ

## モジュール構成

- `src/main.rs` — エントリポイント。CLI パース → git バージョンチェック → サブコマンドへのディスパッチ
- `src/cli.rs` — clap derive による CLI 定義（`Commands` enum がサブコマンド一覧）
- `src/commands/` — 各サブコマンドの実装。各モジュールが `pub fn run()` を公開
- `src/config.rs` — `.ou/settings.toml` + `.ou/settings.local.toml` の読み込み・マージロジック
- `src/git/` — Git 操作の抽象化レイヤー
  - `executor.rs` — `GitExecutor` trait（`std::process::Command` のラッパー）
  - `runner.rs` — `GitRunner<E: GitExecutor>` が実際の git コマンド呼び出しとパース処理を担当
  - `types.rs` — `Worktree`, `Branch`, `MergeStatus`, `CommandOutput` 等のデータ型
- `src/fs.rs` — `FileSystem` trait + `OsFileSystem` 実装 + テスト用 `MockFileSystem`
- `src/symlink.rs` — glob パターンベースの symlink 作成
- `src/multiplexer/` — ターミナルマルチプレクサ抽象化（現在 WezTerm のみ実装）
- `src/tui/` — ratatui ベースの TUI ダッシュボード
- `src/error.rs` — `OuError` enum（thiserror ベース）
- `src/result.rs` — 出力フォーマット用の `FormatResult` 型

## 設計パターン

**trait ベースの DI**: `GitExecutor` と `FileSystem` を trait として定義し、テストではモック実装に差し替え可能。コマンド関数は `&GitRunner<E>`, `&dyn FileSystem`, `&Config` を引数に取る。

**インテグレーションテスト**: `tests/common/mod.rs` に共通ヘルパー（`setup_git_repo()`, `ou_cmd()`, `require_git!` マクロ）を集約。テストでは実際の git リポジトリを tempdir に作成して CLI バイナリを実行する。

## Rust エディション

`edition = "2024"` を使用。`let chains`（`if let Some(x) = ... && ...`）等の新構文を活用している。
