# ou

Git worktree 管理 CLI ツール。worktree の作成・削除・同期・WezTerm 連携を1コマンドに集約します。

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

## 使い方

### `ou init`

`.ou/settings.toml` を初期化する。

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

## 設定

`ou init` で `.ou/settings.toml` が生成される。
個人設定は `.ou/settings.local.toml`（gitignore 対象）に記載。スカラー値は local 優先でマージされる。

### 設定項目

#### `worktree_destination_base_dir`

- **型**: string（任意）
- **デフォルト**: `"../{リポジトリ名}-worktrees"`
- worktree の作成先ディレクトリ。相対パスはリポジトリルート基準で解決される。未指定時はリポジトリの親ディレクトリに `{リポジトリ名}-worktrees` を自動作成。

#### `default_source`

- **型**: string（任意）
- **デフォルト**: `"main"`
- `ou add` で新しい worktree を作る際のベースブランチ。`--source` オプションで上書き可能。`ou init` 実行時にリポジトリのデフォルトブランチが自動設定される。

#### `symlinks`

- **型**: string の配列
- **デフォルト**: `[".env", ".envrc", ".tool-versions"]`
- worktree 作成時に元リポジトリからシンボリックリンクを張るファイル。glob パターン対応。`settings.local.toml` で指定した場合は完全に上書きされる。

#### `extra_symlinks`

- **型**: string の配列
- **デフォルト**: `[]`
- `symlinks` に追加するシンボリックリンク。glob パターン対応。`settings.local.toml` の値はベース設定とマージされ、重複は自動排除される。

#### `init_submodules`

- **型**: bool
- **デフォルト**: `false`
- worktree 作成時にサブモジュールを自動初期化するか。`ou add --init-submodules` でコマンド実行時にも指定可能。

#### `submodule_reference`

- **型**: bool
- **デフォルト**: `false`
- サブモジュール初期化時に参照モード（`--reference`）を使用するか。

#### `[wezterm]` セクション

##### `auto_open`

- **型**: bool
- **デフォルト**: `false`
- `ou add` で worktree 作成後、自動的に WezTerm の新しいタブで開くか。WezTerm 内で実行時のみ有効。

##### `tab_title_template`

- **型**: string（任意）
- **デフォルト**: `"{name}"`
- WezTerm タブのタイトルテンプレート。`{name}` がブランチ名に置換される。

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
```

### ローカル設定

`.ou/settings.local.toml` は `.gitignore` に含まれ、個人環境固有の設定を記述する。スキーマは `settings.toml` と同一。

マージルール:
- **スカラー値**（`worktree_destination_base_dir`, `default_source`）: local に値があれば上書き
- **`symlinks`**: local に指定があれば完全に置き換え
- **`extra_symlinks`**: ベース設定とマージされ、重複は自動排除
- **`init_submodules` / `submodule_reference`**: local で `true` にすると有効化（`false` では上書きされない）
- **`[wezterm]`**: local に指定があればセクションごと置き換え

```toml
# .ou/settings.local.toml の例
extra_symlinks = [".env.local"]
init_submodules = true

[wezterm]
auto_open = true
```

## ダッシュボード

`ou dashboard` で TUI を起動:

- `j`/`k`: ナビゲーション
- `Enter`: WezTerm タブで開く
- `d`: worktree 削除
- `r`: リフレッシュ
- `q`: 終了
