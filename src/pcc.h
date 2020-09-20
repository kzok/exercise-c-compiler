#ifndef _9CC_H_IS_INCLUDED
#define _9CC_H_IS_INCLUDED

#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>
#include <string.h>
#include <assert.h>

#include "./vector.h"

#ifdef ENTRYPOINT
#  define EXTERN extern
#else
#  define EXTERN
#endif

#ifdef DEBUG
# define DEBUGF(...) \
  do {fprintf(stderr, "[DEBUG] ");fprintf(stderr, __VA_ARGS__);fflush(stderr);} while(0);
#else
# define DEBUGF(fmt, ...)
#endif

/**
 * tokenizer.c
 */

// トークンの種類
#define TOKEN_KIND_MAP(XX) \
  XX(TK_SIGN) /** 記号 */ \
  XX(TK_IDENT) /** 識別子 */ \
  XX(TK_NUM) /** 整数トークン */ \
  XX(TK_EOF) /** 入力の終わりを表すトークン */ \
  XX(TK_RETURN) /** リターン文 */ \
  XX(TK_IF) /** if */ \
  XX(TK_ELSE) /** else */ \
  XX(TK_WHILE) /** while */ \
  XX(TK_FOR) /** for */ \

typedef enum {
#define XX(name) name,
  TOKEN_KIND_MAP(XX)
#undef XX
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

Token *tokenize(char* p);

/**
 * parser.c
 */

// 抽象構文木のノードの種類
#define NODE_KIND_MAP(XX) \
   XX(ND_EQ) /** == */ \
   XX(ND_NE) /** != */ \
   XX(ND_LTE) /** <= */ \
   XX(ND_LT) /** < */ \
   XX(ND_ADD) /** + */ \
   XX(ND_SUB) /** - */ \
   XX(ND_MUL) /** * */ \
   XX(ND_DIV) /** / */ \
   XX(ND_ASSIGN) /** 代入 */ \
   XX(ND_LVAR) /** ローカル変数 */ \
   XX(ND_NUM) /** 整数 */ \
   XX(ND_RETURN) /** リターン文 */ \
   XX(ND_IF) /** if 文 */ \
   XX(ND_WHILE) /** while 文 */ \
   XX(ND_FOR) /** for 文 */ \
   XX(ND_BLOCK) /** block */ \

typedef enum {
#define XX(name) name,
  NODE_KIND_MAP(XX)
#undef XX
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

  // 制御構文用の子ノード
  // "if" "("" cond ")" then "else" els
  // "while" "(" cond ")" then
  // "for" "(" init ";" cond ";" inc ")" then
  Node *cond;
  Node *then;
  Node *els;
  Node *init;
  Node *inc;

  // ND_BLOCK
  Vector* children;
};

void program();

// ローカル変数の型
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
 * codegen.c
 */

void codegen();

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

static inline const char* token_kind_str(TokenKind kind) {
  switch (kind) {
#define XX(name) case name: return #name;
    TOKEN_KIND_MAP(XX)
#undef XX
    default: return "<unknown>";
  }
}

static inline const char* node_kind_str(NodeKind kind) {
  switch (kind) {
#define XX(name) case name: return #name;
    NODE_KIND_MAP(XX)
#undef XX
    default: return "<unknown>";
  }
}

// エラーを報告するための関数
// printf と同じ引数を取る
EXTERN inline void error_at(const char* const loc, const char* const fmt, ...) {
  assert(loc != NULL);
  assert(fmt != NULL);

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

#undef EXTERN
#endif // #ifndef 9CC_H_INCLUDED
