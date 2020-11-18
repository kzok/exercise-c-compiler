# C コンパイラ演習

[![CircleCI](https://circleci.com/gh/kzok/exercise-c-compiler.svg?style=shield)](https://circleci.com/gh/kzok/exercise-c-compiler)

[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)の演習用リポジトリ

※ ステップ 15 まで実装してから C 言語から Rust に書き換えました

- [std - Rust](https://doc.rust-lang.org/std/index.html)

## 参考資料

- [chibicc](https://github.com/rui314/chibicc/)
  - [branch: reference](https://github.com/rui314/chibicc/commits/reference)
  - [branch: historical/old](https://github.com/rui314/chibicc/commits/historical/old)

## コマンド

※ Linux/x86-64 でのみ動きます
※ vscode を使う場合は拡張機能 "Remote - Containers" で開発環境を整えられます

## 文法

```
program    = function*
function   = ident "(" (ident ("," ident)*)? ")" "{" stmt* "}"
stmt       = "return" expr ";"
           | "{" stmt* "}"
           | "if" "(" expr ")" stmt ("else" stmt)?
           | "while" "(" expr ")" stmt
           | "for" "(" expr? ";" expr? ";" expr? ")" stmt
           | expr ";"
expr       = assign
assign     = equality ("=" assign)?
equality   = relational ("==" relational | "!=" relational)*
relational = add ("<" add | "<=" add | ">" add | ">=" add)*
add        = mul ("+" mul | "-" mul)*
mul        = unary ("*" unary | "/" unary)*
unary      = ("+" | "-")? primary
           | "*" unary
           | "&" unary
primary    = num | ident args? | "(" expr ")"
args       = "(" (assign ("," assign)*)? ")"
```
