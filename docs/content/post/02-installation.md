---
title: 'インストール方法'
date: 2026-06-12T10:10:00+09:00
lastmod: 2026-06-12T10:10:00+09:00
weight: 2
categories:
  - はじめに
tags:
  - expls
  - インストール
toc: true
---

expls は Rust で書かれています。導入方法はかんたんです。

## 1. Rust を用意する

まだ Rust を入れていない場合は、[rustup](https://rustup.rs/) を使うのが
いちばん簡単です。次のコマンドを実行します。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

インストール後、ターミナルを開き直して、入っているか確認します。

```bash
cargo --version
# => cargo 1.xx.x のように表示されればOK
```

## 2. ソースコードを取得してビルドする

リポジトリを取得し、リリースビルドを作成します。

```bash
git clone https://github.com/kaikai-kitan/expls.git
cd expls
cargo build --release
```

ビルドが終わると、実行ファイルが `target/release/expls` に作られます。

```bash
./target/release/expls
```

## 3. どこからでも使えるようにする（任意）

毎回パスを打つのが面倒なら、`cargo install` でユーザー環境にインストールできます。

```bash
cargo install --path .
```

これで、どのフォルダにいても `expls` だけで実行できるようになります。

```bash
expls
```

## うまくいかないときは

- `cargo: command not found` と出る → Rust が入っていません。手順1からやり直してください。
- 色が表示されない → お使いのターミナルが 24bit カラー（True Color）に対応しているか確認してください。最近のターミナル（iTerm2、Windows Terminal、VS Code のターミナルなど）はほぼ対応しています。

## 次に読む

- [チュートリアル]({{< ref "03-tutorial" >}}) で基本的な使い方を学びましょう。
