# C コンパイラ演習

[![CircleCI](https://circleci.com/gh/kzok/practice-c-compiler.svg?style=shield)](https://circleci.com/gh/kzok/practice-c-compiler)

[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)の演習用リポジトリ

## 目的

- 実際に作ってみて構文解析（≒AST）のしくみを理解する
- アセンブリ言語に対する苦手意識を減らす

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

## 現時点での文法

```
program    = stmt*
stmt       = expr ";"
           | "if" "(" expr ")" stmt ("else" stmt)?
           | "return" expr ";"
expr       = assign
assign     = equality ("=" assign)?
equality   = relational ("==" relational | "!=" relational)*
relational = add ("<" add | "<=" add | ">" add | ">=" add)*
add        = mul ("+" mul | "-" mul)*
mul        = unary ("*" unary | "/" unary)*
unary      = ("+" | "-")? primary
primary    = num | ident | "(" expr ")"
```

## TODO

- return のトークナイザを関数化
- if 文の追加
  - if のトークンを追加
  - if のパーサを追加
- else 句の追加

- 大規模リファクタリング？
  - グローバル変数無くす
  - ユニットテスト書く

- 開発環境を vscode dev container に移す
