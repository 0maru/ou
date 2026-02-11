# ou

Git worktree 管理 CLI ツール。worktree の作成・削除・同期・WezTerm 連携を1コマンドに集約します。

## インストール

```bash
cargo install --path .
```

## 使い方

```bash
ou init                              # .ou/settings.toml を初期化
ou add <name>                        # worktree + ブランチ + symlink を一括作成
    --source <branch>                # ベースブランチ（デフォルト: 設定値）
    --carry                          # 未コミット変更をstash経由で移動
    --lock                           # 新worktreeをロック
    --reason <text>                  # ロック理由
    --init-submodules                # サブモジュール初期化
ou list                              # worktree一覧表示
    --quiet / -q                     # パスのみ出力（fzf等へのパイプ用）
ou remove <branch>...                # worktreeとブランチを削除
    -f                               # 強制: 未コミット変更があっても削除
    -ff                              # 強制: ロック中でも削除
ou clean                             # マージ済み/upstream-goneを一括削除
    --check                          # ドライラン
ou sync                              # symlinkとサブモジュールを同期
    --all                            # 全worktreeに同期
    --source <worktree>              # 同期元worktree
ou open                              # worktree選択 → WezTermタブで開く
ou dashboard                         # TUIダッシュボード起動
```

## 設定

`ou init` で生成される `.ou/settings.toml`:

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

個人設定は `.ou/settings.local.toml`（gitignore 対象）に記載。同一スキーマでスカラー値は local 優先。

## ダッシュボード

`ou dashboard` で TUI を起動:

- `j`/`k`: ナビゲーション
- `Enter`: WezTerm タブで開く
- `d`: worktree 削除
- `r`: リフレッシュ
- `q`: 終了
