---
description: ou で worktree を作成し、plan を立てて実装する開発ワークフロー
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, EnterPlanMode, ExitPlanMode, AskUserQuestion, Task
---

# Worktree 開発ワークフロー

あなたは `ou` CLI を使った worktree ベースの機能開発を支援するエージェントです。
以下のワークフローに従って、worktree の作成 → 実装計画 → コード実装を一貫して行います。

## 引数

`$ARGUMENTS` には以下の形式で入力されます:

```
<ブランチ名> [実装内容の説明]
```

例:
- `feat/add-status-command ステータス表示コマンドを追加`
- `fix/symlink-error symlink 作成時のエラーハンドリングを改善`

引数が空の場合は、ユーザーにブランチ名と実装内容を質問してください。

## ワークフロー

### Step 1: 引数のパース

`$ARGUMENTS` からブランチ名と実装内容の説明を分離します。
最初のスペースまでがブランチ名、それ以降が説明です。

### Step 2: Worktree の作成

`ou add <ブランチ名>` を実行して worktree を作成します。

- 作成前に `ou list -q` で既存の worktree を確認
- 既に同名の worktree がある場合はユーザーに確認
- `--source` オプションが必要か確認（デフォルトブランチ以外からの派生）

作成後、`ou list` で worktree のパスを取得し、以降の作業ディレクトリとして使用します。

### Step 3: 実装計画（Plan Mode）

EnterPlanMode を使用して実装計画を立てます。

計画では以下を含めてください:
- 変更対象のファイル一覧
- 各ファイルの変更内容の概要
- 新規作成が必要なファイル
- テストの方針
- 実装の順序（依存関係を考慮）

ユーザーが計画を承認するまで、実装には進みません。

### Step 4: 実装

計画が承認されたら、worktree ディレクトリ内でコードを実装します。

- worktree のパスを基準に絶対パスでファイルを操作
- 実装後は `cargo fmt` と `cargo clippy` でコード品質を確認
- テストを作成・実行して動作を検証

### Step 5: 完了報告

実装完了後、以下をまとめて報告します:
- 変更したファイルの一覧
- 実装内容の要約
- テスト結果
- 次のステップの提案（PR 作成、追加テストなど）

## 注意事項

- worktree 内での作業であることを常に意識し、正しいディレクトリで操作すること
- `cargo test`、`cargo clippy` は worktree ディレクトリで実行すること
- コミットメッセージは日本語で記述すること（プロジェクトの慣習に従う）
