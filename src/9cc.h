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

/**
 * 字句解析
 */

// トークンの種類
typedef enum {
  // 記号
  TK_RESERVED,
  // 整数トークン
  TK_NUM,
  // 入力の終わりを表すトークン
  TK_EOF,
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
  // 整数
  ND_NUM,
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
};

/**
 * グローバル変数
 */

// 現在着目しているトークン
EXTERN Token *g_token;
// 入力プログラム
EXTERN char *g_user_input;

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

Node *expr();

/**
 * prototype of codegen.c
 */

void gen(Node *node);

#undef EXTERN
#endif // #ifndef 9CC_H_INCLUDED
