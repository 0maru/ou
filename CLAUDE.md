# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> 共通指示事項は ~/.claude/CLAUDE.md を参照

## プロジェクト概要

`ou` は Git worktree 管理 CLI ツール。worktree の作成・削除・同期・WezTerm 連携を1コマンドに集約する Rust 製 CLI。

## 開発コマンド

```bash
cargo build                        # ビルド
cargo test                         # 全テスト実行（ユニット + インテグレーション）
cargo test --test integration_test # 特定のインテグレーションテストファイル
cargo test test_merge_             # テスト名のパターンマッチで実行
cargo fmt --all -- --check         # フォーマットチェック
cargo fmt --all                    # フォーマット適用
cargo clippy --all-targets --all-features -- -D warnings  # lint
cargo install --path .             # ローカルインストール
```

**注意**: `cargo test --lib` はライブラリターゲットがない（バイナリクレートのみ）ため使用不可。ユニットテストも `cargo test` で実行される。

## コミットメッセージ

日本語で記述する。
