# ou

[![CI](https://github.com/0maru/ou/actions/workflows/ci.yml/badge.svg)](https://github.com/0maru/ou/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Git worktree 管理 CLI ツール。worktree の作成・削除・同期・WezTerm 連携を1コマンドに集約します。

## 特徴

- worktree + ブランチ + symlink を一括作成
- 未コミット変更の carry / sync
- マージ済み worktree の一括クリーンアップ
- symlink / サブモジュールの自動同期
- post_add フックによるカスタムコマンド実行
- WezTerm タブ連携 & TUI ダッシュボード

## インストール

### Homebrew

```bash
brew tap 0maru/formulae
brew install ou
```

### ソースからビルド

```bash
cargo install --path .
```

## クイックスタート

```bash
# 1. リポジトリで初期化
ou init

# 2. worktree を作成
ou add feat/my-feature

# 3. 一覧表示
ou list

# 4. マージ済みを一括削除
ou clean
```

## コマンド

### `ou init`

`.ou/settings.toml` を初期化する。リポジトリのデフォルトブランチが `default_source` に自動設定される。

### `ou add <name>`

worktree + ブランチ + symlink を一括作成する。

| オプション | 説明 |
|---|---|
| `--source <branch>` | ベースブランチ（デフォルト: 設定値） |
| `--carry` | 未コミット変更を stash 経由で移動 |
| `--sync` | 未コミット変更を両 worktree にコピー |
| `--file <path>` | carry/sync の対象ファイルを限定（複数指定可） |
| `--lock` | 新 worktree をロック |
| `--reason <text>` | ロック理由 |
| `--init-submodules` | サブモジュール初期化 |
| `--submodule-reference` | サブモジュール参照モードを使用 |

### `ou list`

worktree 一覧を表示する。

| オプション | 説明 |
|---|---|
| `-q`, `--quiet` | パスのみ出力（fzf 等へのパイプ用） |

### `ou remove <branch>...`

worktree とブランチを削除する。

| オプション | 説明 |
|---|---|
| `-f` | 未コミット変更があっても削除 |
| `-ff` | ロック中でも削除 |

### `ou clean`

マージ済み/upstream-gone の worktree を一括削除する。

| オプション | 説明 |
|---|---|
| `--check` | ドライラン |

### `ou sync`

symlink とサブモジュールを同期する。

| オプション | 説明 |
|---|---|
| `--all` | 全 worktree に同期 |
| `--source <worktree>` | 同期元 worktree |

### `ou open`

worktree を選択して WezTerm タブで開く。

### `ou dashboard`

TUI ダッシュボードを起動する。

| キー | 操作 |
|---|---|
| `j` / `k` | ナビゲーション |
| `Enter` | WezTerm タブで開く |
| `d` | worktree 削除 |
| `r` | リフレッシュ |
| `q` | 終了 |

## 設定

`ou init` で `.ou/settings.toml` が生成される。
個人設定は `.ou/settings.local.toml`（gitignore 対象）に記載。スカラー値は local 優先でマージされる。

### 設定項目

| キー | 型 | デフォルト | 説明 |
|---|---|---|---|
| `worktree_destination_base_dir` | string? | `なし` | worktree の作成先ディレクトリ。未指定時は `.ou/worktrees` |
| `default_source` | string? | `"main"` | `ou add` のベースブランチ |
| `symlinks` | string[] | `[".env", ".envrc", ".tool-versions"]` | worktree 作成時にシンボリックリンクを張るファイル（glob 対応） |
| `extra_symlinks` | string[] | `[]` | `symlinks` に追加するリンク（glob 対応、マージ時に重複排除） |
| `init_submodules` | bool | `false` | worktree 作成時にサブモジュールを自動初期化 |
| `submodule_reference` | bool | `false` | サブモジュール初期化時に参照モードを使用 |

#### `[wezterm]` セクション

| キー | 型 | デフォルト | 説明 |
|---|---|---|---|
| `auto_open` | bool | `false` | `ou add` 後に自動的に WezTerm タブで開く |
| `tab_title_template` | string? | `"{name}"` | タブタイトルテンプレート（`{name}` がブランチ名に置換） |

#### `[hooks]` セクション

| キー | 型 | デフォルト | 説明 |
|---|---|---|---|
| `post_add` | string[] | `[]` | `ou add` 完了後に実行するコマンド |

`post_add` フックでは以下のプレースホルダが使用可能:

- `{worktree_path}` — 作成された worktree のパス

```toml
[hooks]
post_add = [
  "echo {worktree_path}",
  "touch {worktree_path}/.ready",
]
```

### 設定例（完全版）

```toml
worktree_destination_base_dir = "../myproject-worktrees"
default_source = "main"
symlinks = [".env", ".envrc", ".tool-versions"]
extra_symlinks = []
init_submodules = false
submodule_reference = false

[wezterm]
auto_open = false
tab_title_template = "{name}"

[hooks]
post_add = []
```

### ローカル設定

`.ou/settings.local.toml` は `.gitignore` に含まれ、個人環境固有の設定を記述する。スキーマは `settings.toml` と同一。

マージルール:

| 設定 | ルール |
|---|---|
| スカラー値（`worktree_destination_base_dir`, `default_source`） | local に値があれば上書き |
| `symlinks` | local に指定があれば完全に置き換え |
| `extra_symlinks` | ベース設定とマージ（重複自動排除） |
| `init_submodules` / `submodule_reference` | local で `true` にすると有効化（`false` では上書きされない） |
| `[wezterm]` | local に指定があればセクションごと置き換え |
| `[hooks]` | local に指定があればセクションごと置き換え |

```toml
# .ou/settings.local.toml の例
extra_symlinks = [".env.local"]
init_submodules = true

[wezterm]
auto_open = true
```

## ライセンス

[MIT](LICENSE)
