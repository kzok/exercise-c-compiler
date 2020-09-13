# C コンパイラ演習

[![CircleCI](https://circleci.com/gh/kzok/exercise-c-compiler.svg?style=shield)](https://circleci.com/gh/kzok/exercise-c-compiler)

[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)の演習用リポジトリ

## 目的

- 実際に作ってみて構文解析（≒AST）のしくみを理解する
- アセンブリ言語に対する苦手意識を減らす

## 使い方

※ Linux/x86-64 でのみ動きます
※ vscode を使う場合は拡張機能 "Remote - Containers" で開発環境を整えられます

- ビルド
  ```bash
  ./build.sh
  ```

  - `./out/pcc` にファイルが出来上がります

- テスト
  ```bash
  ./test.sh
  ```

  - テストを実行します

## 現時点での文法

```
program    = stmt*
stmt       = expr ";"
           | "if" "(" expr ")" stmt
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

- codegen.c のリファクタリング
  - ポインタ参照前に assert(p != NULL); を追加
  - gen(); の分割
- else 句の追加
