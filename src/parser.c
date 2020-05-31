#include "9cc.h"

#include <stdbool.h>
#include <string.h>

// 次のトークンが期待している記号のときには、トークンを一つ読み進める。
// それ以外の場合にはエラーを報告する。
static void expect(char *op) {
  if (
    g_token->kind != TK_SIGN ||
    g_token->len != strlen(op) ||
    memcmp(g_token->str, op, g_token->len)
  ) {
    error_at(g_token->str, "'%s' ではありません", op);
  }
  g_token = g_token->next;
}

static bool at_eof() {
  return g_token->kind == TK_EOF;
}

// 次のトークンが期待している記号のときには、トークンを一つ読み進めて真を返す。
// それ以外の場合には偽を返す。
static bool consume_as_sign(char *op) {
  if (
      g_token->kind != TK_SIGN ||
      g_token->len != strlen(op) ||
      memcmp(g_token->str, op, g_token->len)
  ) {
    return false;
  }
  g_token = g_token->next;
  return true;
}

// 次のトークンが期待した種類のものだったときには、
// トークンを一つ読み進めて現在のトークンを返す
// そうでなければ NULL ポインタを返す
static Token *consume_token_kind(TokenKind kind) {
  if (g_token->kind != kind) {
    return NULL;
  }
  Token *current = g_token;
  g_token = g_token->next;
  return current;
}

// 次のトークンが数値の場合、トークンを一つ読み進めてその数値を返す。
// それ以外の場合にはエラーを報告する。
static int expect_number() {
  if (g_token->kind != TK_NUM) {
    error_at(g_token->str, "数ではありません");
  }
  int val = g_token->val;
  g_token = g_token->next;
  return val;
}

static Node *new_node(NodeKind kind, Node *lhs, Node*rhs) {
  Node *node = calloc(1, sizeof(Node));
  node->kind = kind;
  node->lhs = lhs;
  node->rhs = rhs;
  return node;
}

static Node *new_node_num(int val) {
  Node *node = calloc(1, sizeof(Node));
  node->kind = ND_NUM;
  node->val = val;
  return node;
}

// 変数を名前で検索する。見つからなかった場合は NULL を返す。
static LVar *find_lvar(Token *tok) {
  for (LVar *var = g_locals; var; var = var->next) {
    if (var->len == tok->len && !memcmp(tok->str, var->name, var->len)) {
      return var;
    }
  }
  return NULL;
}

/**
 * Syntax rules
 */

static Node *expr();

static Node *primary() {
  // トークンが "(" ならば "(" expr ")" のはず
  if (consume_as_sign("(")) {
    Node *node = expr();
    expect(")");
    return node;
  }
  Token *token = consume_token_kind(TK_IDENT);
  if (token) {
    Node *node = calloc(1, sizeof(Node));
    node->kind = ND_LVAR;

    LVar *lvar = find_lvar(token);
    if (lvar) {
      node->offset = lvar->offset;
    } else {
      lvar = calloc(1, sizeof(LVar));
      lvar->next = g_locals;
      lvar->name = token->str;
      lvar->len = token->len;
      // 今は int 型しかないので 8 バイト固定
      lvar->offset = g_locals == NULL ? 0 : g_locals->offset + 8;
      node->offset = lvar->offset;
      g_locals = lvar;
    }
    return node;
  }
  // そうでなければ数値または識別子のはず
  return new_node_num(expect_number());
}

static Node *unary() {
  if (consume_as_sign("+")) {
    return primary();
  } else if (consume_as_sign("-")) {
    return new_node(ND_SUB, new_node_num(0), primary());
  } else {
    return primary();
  }
}

static Node *mul() {
  Node *node = unary();

  for (;;) {
    if (consume_as_sign("*")) {
      node = new_node(ND_MUL, node, unary());
    } else if (consume_as_sign("/")) {
      node = new_node(ND_DIV, node, unary());
    } else {
      return node;
    }
  }
}

static Node *add() {
  Node *node = mul();

  for (;;) {
    if (consume_as_sign("+")) {
      node = new_node(ND_ADD, node, mul());
    } else if (consume_as_sign("-")) {
      node = new_node(ND_SUB, node, mul());
    } else {
      return node;
    }
  }
}

static Node *relational() {
  Node *node = add();

  for (;;) {
    if (consume_as_sign("<=")) {
      node = new_node(ND_LTE, node, add());
    } else if (consume_as_sign(">=")) {
      node = new_node(ND_LTE, add(), node);
    } else if (consume_as_sign("<")) {
      node = new_node(ND_LT, node, add());
    } else if (consume_as_sign(">")) {
      node = new_node(ND_LT, add(), node);
    } else {
      return node;
    }
  }
}

static Node *equality() {
  Node *node = relational();

  for (;;) {
    if (consume_as_sign("==")) {
      node = new_node(ND_EQ, node, relational());
    } else if (consume_as_sign("!=")) {
      node = new_node(ND_NE, node, relational());
    } else {
      return node;
    }
  }
}

static Node *assign() {
  Node *node = equality();

  for (;;) {
    if (consume_as_sign("=")) {
      node = new_node(ND_ASSIGN, node, assign());
    } else {
      return node;
    }
  }
}

static Node *expr() {
  return assign();
}

static Node *stmt() {
  Node *node;
  Token *token = consume_token_kind(TK_RETURN);
  if (token) {
    node = new_node(ND_RETURN, expr(), NULL);
  } else {
    node = expr();
  }
  expect(";");
  return node;
}

void program() {
  int i = 0;
  while (!at_eof()) {
    g_code[i] = stmt();
    i += 1;
  }
  g_code[i] = NULL;
}
