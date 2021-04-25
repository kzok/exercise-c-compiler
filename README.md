# C コンパイラ演習

[![CircleCI](https://circleci.com/gh/kzok/exercise-c-compiler.svg?style=shield)](https://circleci.com/gh/kzok/exercise-c-compiler)

[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)の演習用リポジトリ

- ※ Linux/x86-64 でのみ動きます
- ※ vscode を使う場合は拡張機能 "Remote - Containers" で開発環境を整えられます

## 参考資料

- [chibicc](https://github.com/rui314/chibicc/)
  - [branch: historical/old](https://github.com/rui314/chibicc/commits/historical/old) 最初こっちのブランチ見ながら書いてました
  - [branch: reference](https://github.com/rui314/chibicc/commits/reference) 途中からこっちのブランチ見るようになりました。
- [std - Rust](https://doc.rust-lang.org/std/index.html)

## 文法

```
program     = (global-var | function)*
global-var  = basetype ident ("[" num "]")* ";"
function    = basetype ident "(" params? ")" "{" stmt* "}"
params      = param ("," param)*
param       = basetype ident ("[" num "]")*
stmt        = "return" expr ";"
            | "{" stmt* "}"
            | "if" "(" expr ")" stmt ("else" stmt)?
            | "while" "(" expr ")" stmt
            | "for" "(" expr? ";" expr? ";" expr? ")" stmt
            | declaretion
            | expr ";"
declaretion = basetype ident ("[" num "]")* ("=" expr)? ";"
expr        = assign
assign      = equality ("=" assign)?
equality    = relational ("==" relational | "!=" relational)*
relational  = add ("<" add | "<=" add | ">" add | ">=" add)*
add         = mul ("+" mul | "-" mul)*
mul         = unary ("*" unary | "/" unary)*
unary       = ("+" | "-")? primary
            | ("*" | "&") unary
            | postfix
postfix     = primary ("[" expr "]")*
primary     = num
            | "sizeof" unary
            | funcall
            | "(" expr ")"
funcall     = ident ("(" (assign ("," assign)*)? ")")?
basetype    = "int" "*"*
```
