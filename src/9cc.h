#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>
#include <string.h>

#ifndef _9CC_H_IS_INCLUDED
#define _9CC_H_IS_INCLUDED

#ifdef ENTRYPOINT
#  define EXTERN extern
#else
#  define EXTERN
#endif

#ifdef DEBUG
# define DEBUGF(...) do {fprintf(stderr, __VA_ARGS__);fflush(stderr);} while(0);
#else
# define DEBUGF(fmt, ...)
#endif

/**
 * 字句解析
 */

// トークンの種類
typedef enum {
  // 記号
  TK_SIGN,
  // 識別子
  TK_IDENT,
  // 整数トークン
  TK_NUM,
  // 入力の終わりを表すトークン
  TK_EOF,
  // リターン文
  TK_RETURN,
} TokenKind;

// トークン型
typedef struct Token Token;
struct Token {
  // トークンの型
  TokenKind kind;
  // 次の入力トークン
  Token *next;
  // kind が TK_NUM の場合、その数値
  int val;
  // トークン文字列
  char *str;
  // 文字列長
  int len;
};

/**
 * 構文解析
 */

// 抽象構文木のノードの種類
typedef enum {
  // ==
  ND_EQ,
  // !=
  ND_NE,
  // <=
  ND_LTE,
  // <
  ND_LT,
  // +
  ND_ADD,
  // -
  ND_SUB,
  // *
  ND_MUL,
  // / 
  ND_DIV,
  // 代入
  ND_ASSIGN,
  // ローカル変数
  ND_LVAR,
  // 整数
  ND_NUM,
  // リターン文
  ND_RETURN,
} NodeKind;

// 抽象構文木のノードの型
typedef struct Node Node;
struct Node {
  // ノードの型
  NodeKind kind;
  // 左辺
  Node *lhs;
  // 右辺
  Node *rhs;
  // kind が ND_NUM の場合のみ使う
  int val;
  // kind が ND_LVAR の場合のみ使う
  int offset;
};

/**
 * ローカル変数の型
 */
typedef struct LVar LVar;
struct LVar {
  // 次の変数か NULL
  LVar *next;
  // 変数の名前
  char *name;
  // 名前の長さ
  int len;
  // RBP からのオフセット
  int offset;
};

/**
 * グローバル変数
 */

// 現在着目しているトークン
EXTERN Token *g_token;
// 入力プログラム
EXTERN char *g_user_input;
// 生成された構文木
EXTERN Node *g_code[100];
// ローカル変数
EXTERN LVar *g_locals;

/**
 * インライン関数
 */

// エラーを報告するための関数
// printf と同じ引数を取る
EXTERN inline void error_at(char *loc, char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);

  int pos = loc - g_user_input;
  fprintf(stderr, "%s\n", g_user_input);
  // pos 個の空白を出力
  fprintf(stderr, "%*s", pos, "");
  fprintf(stderr, "^ ");
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");
  exit(1);
}

/**
 * prototype of tokenizer.c
 */

Token *tokenize(char *p);

/**
 * prototype of parser.c
 */

void program();

/**
 * prototype of codegen.c
 */

void gen(Node *node);

#undef EXTERN
#endif // #ifndef 9CC_H_INCLUDED
