# expls (エクスプレス)

[![License: MIT](https://img.shields.io/badge/License-MIT-orange)](https://github.com/kaikai-kitan/expls/blob/main/LICENSE)
[![Coverage Status](https://coveralls.io/repos/github/kaikai-kitan/expls/badge.svg?branch=main)](https://coveralls.io/github/kaikai-kitan/expls?branch=main)

## Description

`expls` は、標準の `ls` を拡張したファイル一覧表示コマンドです。
ファイルの **拡張子** に応じて色分けし、さらに同じ拡張子のファイル同士では
**更新日時** に基づいて色をグラデーション表示します。更新が新しいファイルほど
濃く鮮やかに、古いファイルほど薄く淡く表示されるため、どのファイルが最近
変更されたかを一目で把握できます。

- **origin**: expand × ls
- **author**: Kaido Iwata

## Features

- **拡張子ベースの色分け**
  `.rs`, `.py`, `.js`, `.ts`, `.md`, `.json`, `.html`, `.css`, 画像・動画・音声・
  圧縮ファイルなど、40 種類以上の拡張子カテゴリごとに固有の色相（HSL の hue）を
  割り当てて表示します。`.jsx` と `.js`、`.yaml` と `.yml` のような関連拡張子は
  同系色でグループ化されます。未知の拡張子はグレースケールで表示されます。
- **更新日時によるグラデーション**
  同じ拡張子のファイルを更新日時で並べ、最新のものを濃く（彩度高・明度低）、
  最古のものを薄く（彩度低・明度高）表示します。`--reverse` で順序を反転できます。
- **ディレクトリの再帰表示**
  `--depth` で指定した深さまでサブディレクトリを再帰的に辿り、ネストを
  インデントで表現します。ディレクトリ名は末尾に `/` を付けて表示します。
- **隠しファイルの表示切り替え**
  デフォルトでは `.` で始まるファイル／ディレクトリを非表示にし、`--all` で表示します。
- **24-bit トゥルーカラー出力**
  ANSI エスケープシーケンス（`\x1b[38;2;R;G;Bm`）で色を出力します。
- **シェル補完ファイルの生成**
  隠しオプション `--completions` で、bash / zsh / fish / elvish / powershell 向けの
  補完ファイルを生成します。

## Usage

`expls` は標準の `ls` コマンドのように使用できます。

### デフォルトの実行

オプションを指定せずに実行すると、カレントディレクトリ内のファイル一覧を出力します。
拡張子ごとに異なる色で表示され、更新日時が新しいものほど色が濃く、古いものほど
薄くグラデーション表示されます。

```bash
$ expls
```

### オプション

様々なオプションを組み合わせて表示をカスタマイズできます。

| オプション | 種類 | 説明 |
|------------|------|------|
| `-h, --help` | フラグ | コマンドの使い方（ヘルプ）を出力します。 |
| `-a, --all` | 真偽値フラグ | `.` で始まる隠しファイルや隠しディレクトリも表示します。 |
| `-p, --path <パス>` | 文字列 | 指定したディレクトリ内のファイル一覧を表示します。デフォルトはカレントディレクトリ (`.`)。 |
| `-d, --depth <数値>` | 数値 | サブディレクトリを再帰的に表示する際の深さを指定します。 |
| `--reverse` | 真偽値フラグ | 更新日時のグラデーションの順序を逆にします（古いものを濃く、新しいものを薄く表示）。 |

### 実行例

```bash
# 隠しファイルも含めて、深さ 2 まで再帰表示する
$ expls --all --depth 2

# 指定したディレクトリを表示する
$ expls --path src

# グラデーションを反転する（古いファイルほど濃く表示）
$ expls --reverse
```

### シェル補完ファイルの生成（隠しオプション）

`--completions` を指定すると、カレントディレクトリの `completions/` 以下に
各シェル向けの補完ファイルを生成して終了します（ヘルプには表示されない隠しオプションです）。

```bash
$ expls --completions
# completions/bash/expls, completions/zsh/_expls, completions/fish/expls,
# completions/elvish/expls, completions/powershell/expls が生成される
```

生成した補完ファイルを各シェルに読み込ませることで、`expls` のタブ補完が有効になります。

## Installation

Rust のツールチェイン（`cargo`）が必要です。

```bash
# リポジトリを取得してビルド・インストール
$ git clone https://github.com/kaikai-kitan/expls.git
$ cd expls
$ cargo install --path .
```

または、ビルドのみ行う場合:

```bash
$ cargo build --release
# 生成物: target/release/expls
```

## License

MIT License. 詳細は [LICENSE](LICENSE) を参照してください。
