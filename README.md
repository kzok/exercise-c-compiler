# C コンパイラ演習

[![CircleCI](https://circleci.com/gh/kzok/exercise-c-compiler.svg?style=shield)](https://circleci.com/gh/kzok/exercise-c-compiler)

[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)の演習用リポジトリ

## 参考資料

- [9cc](https://github.com/rui314/9cc)
- [chibicc](https://github.com/rui314/chibicc)

## ビルドおよびテスト

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
function  = ident "(" ")" "{" stmt* "}"
stmt       = expr ";"
           | "{" stmt* "}"
           | "if" "(" expr ")" stmt ("else" stmt)?
           | "while" "(" expr ")" stmt
           | "for" "(" expr? ";" expr? ";" expr? ")" stmt
           | "return" expr ";"
expr       = assign
assign     = equality ("=" assign)?
equality   = relational ("==" relational | "!=" relational)*
relational = add ("<" add | "<=" add | ">" add | ">=" add)*
add        = mul ("+" mul | "-" mul)*
mul        = unary ("*" unary | "/" unary)*
unary      = ("+" | "-")? primary
primary    = num
           | ident callargs?
           | "(" expr ")"
callargs  = "(" (assign ("," assign)*)? ")"
```
