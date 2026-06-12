---
title: '実行例集'
date: 2026-06-12T10:40:00+09:00
lastmod: 2026-06-12T10:40:00+09:00
weight: 5
categories:
  - 使い方
tags:
  - expls
  - 実行例
toc: true
---

ここでは、よく使う場面ごとに expls の実行例をまとめます。
（実際の出力はカラー表示されます。下のテキストは色を除いたイメージです。）

## 例1：いまのフォルダをさっと見る

```bash
expls
```

```text
Cargo.lock
Cargo.toml
LICENSE
README.md
src/
target/
```

拡張子ごとに色分けされ、同じ拡張子のなかでは新しいものほど濃く表示されます。

## 例2：隠しファイルも含めて全部見る

```bash
expls -a
```

```text
.git/
.github/
.gitignore
Cargo.lock
Cargo.toml
LICENSE
README.md
src/
target/
```

`.git` などの隠し項目が増えているのが分かります。

## 例3：別のフォルダを指定して見る

```bash
expls -p ./src
```

```text
main.rs
```

`src` フォルダの中だけを表示しています。移動する必要はありません。

## 例4：サブフォルダの中まで表示する

```bash
expls -d 1
```

```text
Cargo.lock
Cargo.toml
LICENSE
README.md
src/
  main.rs
target/
  CACHEDIR.TAG
  debug/
```

`src/` や `target/` の **中身が1段下げて**表示されます。
`-d 2` にすれば、さらにもう一段深くまで潜ります。

## 例5：グラデーションを逆にする

```bash
expls --reverse
```

新しいものを薄く、古いものを濃く表示します。
「昔から残っているファイルを目立たせたい」ときに使います。

## 例6：オプションを組み合わせる

```bash
# src フォルダを、隠しファイル込みで、2段階潜って表示
expls -a -d 2 -p ./src
```

複数のオプションは自由に組み合わせられます。
短縮形（`-a`, `-d`, `-p`）でも、長い形（`--all`, `--depth`, `--path`）でも構いません。

## 例7：使い方を忘れたとき

```bash
expls --help
```

```text
ls with extension-based colors and modification-time gradients

Usage: expls [OPTIONS]

Options:
  -a, --all            Show hidden files and directories (starting with '.')
  -p, --path <PATH>    Target directory (default: current directory) [default: .]
  -d, --depth <DEPTH>  Recursion depth for subdirectories
      --reverse        Reverse gradient: old = dark, new = light
  -h, --help           Print help
```

## もっと知りたい

- 色がどう決まるのか → [色分けのしくみ]({{< ref "04-algorithm" >}})
- 最初から学びたい → [チュートリアル]({{< ref "03-tutorial" >}})
