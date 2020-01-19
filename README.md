# C コンパイラ演習

[![CircleCI](https://circleci.com/gh/kzok/practice-c-compiler.svg?style=shield)](https://circleci.com/gh/kzok/practice-c-compiler)

[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)の演習用リポジトリ

## 目的

- ブックマーク消化枠（前々から興味があったが手を付けていなかったので）
- 新しい言語・ツールを覚えずに楽しそうなコードを書きたい
  - ~~特別に C 言語が書きたいというわけではない~~

## ゴール

- 実際に作ってみて構文解析（≒AST）のしくみを理解する
- アセンブリ言語に対する苦手意識を減らす
- 楽しくコードを書いて個人開発のモチベーションを上げる

## 使い方

※ Linux/x86-64 で動かして下さい

- 開発環境用の docker container の立ち上げ
  ```bash
  ./docker.sh
  ```

- ビルド
  ```bash
  make
  ```

- テスト
  ```bash
  make test
  ```
