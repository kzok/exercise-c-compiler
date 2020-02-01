#include "9cc.h"

#include <stdbool.h>
#include <string.h>

// 次のトークンが期待している記号のときには、トークンを一つ読み進める。
// それ以外の場合にはエラーを報告する。
static void expect(char *op) {
  if (
    g_token->kind != TK_RESERVED ||
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
static bool consume(char *op) {
  if (
      g_token->kind != TK_RESERVED ||
      g_token->len != strlen(op) ||
      memcmp(g_token->str, op, g_token->len)
  ) {
    return false;
  }
  g_token = g_token->next;
  return true;
}

// 次のトークンが識別子だったときには、トークンを一つ読み進めて現在のトークンを返す
// そうでなければ NULL ポインタを返す
static Token *consume_ident() {
  if (g_token->kind != TK_IDENT) {
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

/**
 * Syntax rules
 */

static Node *expr();

static Node *primary() {
  // トークンが "(" ならば "(" expr ")" のはず
  if (consume("(")) {
    Node *node = expr();
    expect(")");
    return node;
  }
  Token *token = consume_ident();
  if (token) {
    Node *node = calloc(1, sizeof(Node));
    node->kind = ND_LVAR;
    node->offset = (token->str[0] - 'a' + 1) * 8;
    return node;
  }
  // そうでなければ数値または識別子のはず
  return new_node_num(expect_number());
}

static Node *unary() {
  if (consume("+")) {
    return primary();
  } else if (consume("-")) {
    return new_node(ND_SUB, new_node_num(0), primary());
  } else {
    return primary();
  }
}

static Node *mul() {
  Node *node = unary();

  for (;;) {
    if (consume("*")) {
      node = new_node(ND_MUL, node, unary());
    } else if (consume("/")) {
      node = new_node(ND_DIV, node, unary());
    } else {
      return node;
    }
  }
}

static Node *add() {
  Node *node = mul();

  for (;;) {
    if (consume("+")) {
      node = new_node(ND_ADD, node, mul());
    } else if (consume("-")) {
      node = new_node(ND_SUB, node, mul());
    } else {
      return node;
    }
  }
}

static Node *relational() {
  Node *node = add();

  for (;;) {
    if (consume("<=")) {
      node = new_node(ND_LTE, node, add());
    } else if (consume(">=")) {
      node = new_node(ND_LTE, add(), node);
    } else if (consume("<")) {
      node = new_node(ND_LT, node, add());
    } else if (consume(">")) {
      node = new_node(ND_LT, add(), node);
    } else {
      return node;
    }
  }
}

static Node *equality() {
  Node *node = relational();

  for (;;) {
    if (consume("==")) {
      node = new_node(ND_EQ, node, relational());
    } else if (consume("!=")) {
      node = new_node(ND_NE, node, relational());
    } else {
      return node;
    }
  }
}

static Node *assign() {
  Node *node = equality();

  for (;;) {
    if (consume("=")) {
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
  Node *node = expr();
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
