---
description: ou の新バージョンをリリースし、Homebrew formulae を更新する
allowed-tools: Bash, Read, Edit, AskUserQuestion
---

# リリースワークフロー

ou の新バージョンを GitHub Release として公開し、Homebrew formulae を更新するワークフロー。

## 引数

`$ARGUMENTS` にはリリースするバージョン（例: `v0.3.0`）を指定する。
引数が空の場合は、ユーザーにバージョンを質問する。

## 前提条件

- リリース対象のバージョンタグが `0maru/ou` リポジトリにプッシュ済みであること
- `homebrew-formulae` リポジトリが `$HOME/workspaces/github.com/0maru/homebrew-formulae` にクローン済みであること

## ワークフロー

### Step 1: バージョンの確認

`$ARGUMENTS` からバージョンを取得する。`v` プレフィックスがない場合は付与する。

タグが存在するか確認する:

```bash
git tag -l <version>
```

タグが存在しない場合はユーザーに確認する。

### Step 2: リリースノートの準備

前バージョンのタグから現バージョンのタグまでのコミットログを取得する:

```bash
git log <前バージョンタグ>..<バージョンタグ> --oneline
```

コミットログからリリースノートを作成し、ユーザーに確認する。

### Step 3: GitHub Release の作成

```bash
gh release create <version> --repo 0maru/ou --title "<version>" --notes "<リリースノート>"
```

既にリリースが存在する場合はスキップする。

### Step 4: tarball の SHA256 を取得

```bash
gh release download <version> --repo 0maru/ou --archive tar.gz --output /tmp/ou-<version>.tar.gz
shasum -a 256 /tmp/ou-<version>.tar.gz
```

### Step 5: Homebrew formulae を更新

対象ファイル: `$HOME/workspaces/github.com/0maru/homebrew-formulae/Formula/ou.rb`

変更箇所:
- `url` のバージョン部分を新しいバージョンに更新
- `sha256` を Step 4 で取得したハッシュ値に更新

### Step 6: コミット＆プッシュ

```bash
cd $HOME/workspaces/github.com/0maru/homebrew-formulae
git add Formula/ou.rb
git commit -m "ou formula を <version> に更新"
git push
```

### Step 7: 完了報告

以下をまとめて報告する:
- GitHub Release の URL
- Homebrew formulae の変更内容
- 検証方法:
  ```bash
  brew update
  brew upgrade ou
  ou --version
  ```
