#include <ctype.h>
#include <stdlib.h>
#include <stdarg.h>
#include <stdio.h>

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
  TK_RESERVED,  // 記号
  TK_NUM,       // 整数トークン
  TK_EOF,       // 入力の終わりを表すトークン
} TokenKind;

// トークン型
typedef struct Token Token;
struct Token {
  TokenKind kind; // トークンの型
  Token *next;    // 次の入力トークン
  int val;        // kind が TK_NUM の場合、その数値
  char *str;      // トークン文字列
  int len;        // 文字列長
};

/**
 * 構文解析
 */

// 抽象構文木のノードの種類
typedef enum {
  ND_EQ,        // ==
  ND_NE,        // !=
  ND_LTE,       // <=
  ND_LT,        // <
  ND_ADD,       // +
  ND_SUB,       // -
  ND_MUL,       // *
  ND_DIV,       // / 
  ND_NUM,       // 整数
} NodeKind;

// 抽象構文木のノードの型
typedef struct Node Node;
struct Node {
  NodeKind kind;  // ノードの型
  Node *lhs;      // 左辺
  Node *rhs;      // 右辺
  int val;        // kind が ND_NUM の場合のみ使う
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
 * prototype of parser.c
 */

Node *expr();

/**
 * prototype of codegen.c
 */

void gen(Node *node);

#undef EXTERN
#endif // #ifndef 9CC_H_INCLUDED
