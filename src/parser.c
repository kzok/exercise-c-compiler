#include "pcc.h"

#include <stdbool.h>
#include <string.h>

static void seek_token() {
  assert(g_token != NULL);
  g_token = g_token->next;
}

// 次のトークンが期待している記号のときには、トークンを一つ読み進める。
// それ以外の場合にはエラーを報告する。
static void expect(char *op) {
  assert(op != NULL);
  assert(g_token != NULL);

  if (
    g_token->kind != TK_SIGN
    || g_token->len != strlen(op)
    || memcmp(g_token->str, op, g_token->len)
  ) {
    error_at(g_token->str, "'%s' ではありません", op);
  }
  seek_token();
}

static bool at_eof() {
  assert(g_token != NULL);
  return g_token->kind == TK_EOF;
}

// 次のトークンが期待している記号のときには、トークンを一つ読み進めて真を返す。
// それ以外の場合には偽を返す。
static bool consume_as_sign(char *op) {
  assert(op != NULL);

  if (
      g_token->kind != TK_SIGN ||
      g_token->len != strlen(op) ||
      memcmp(g_token->str, op, g_token->len)
  ) {
    return false;
  }
  seek_token();
  return true;
}

// 次のトークンが期待した種類のものだったときには、
// トークンを一つ読み進めて現在のトークンを返す
// そうでなければ NULL ポインタを返す
static Token *consume_token_kind(TokenKind kind) {
  assert(g_token != NULL);

  if (g_token->kind != kind) {
    return NULL;
  }
  Token *current = g_token;
  seek_token();
  return current;
}

// 次のトークンが数値の場合、トークンを一つ読み進めてその数値を返す。
// それ以外の場合にはエラーを報告する。
static int expect_number() {
  assert(g_token != NULL);

  if (g_token->kind != TK_NUM) {
    error_at(g_token->str, "数ではありません");
  }
  int val = g_token->val;
  seek_token();
  return val;
}

static Node *new_node(NodeKind kind) {
  Node *node = calloc(1, sizeof(Node));
  node->kind = kind;
  return node;
}

static Node *new_node_binary_ops(NodeKind kind, Node *lhs, Node*rhs) {
  Node *node = new_node(kind);
  node->lhs = lhs;
  node->rhs = rhs;
  return node;
}

static Node *new_node_num(int val) {
  Node *node = new_node(ND_NUM);
  node->val = val;
  return node;
}

// 変数を名前で検索する。見つからなかった場合は NULL を返す。
static LVar *find_lvar(Token *tok) {
  assert(tok != NULL);
  assert(g_locals != NULL);

  for (size_t i = 0; i < g_locals->length; i += 1) {
    LVar *var = vector_at(g_locals, i);
    if (var->len == tok->len && !memcmp(tok->str, var->name, var->len)) {
      return var;
    }
  }
  return NULL;
}

static LVar *new_lvar(Token *token) {
  assert(g_locals != NULL);
  LVar *lvar = calloc(1, sizeof(LVar));
  lvar->name = token->str;
  lvar->len = token->len;
  // 今は int 型しかないので 8 バイト固定
  lvar->offset = vector_empty(g_locals) ? 0 : ((LVar*)vector_last(g_locals))->offset + 8;
  return lvar;
}

/**
 * Syntax rules
 */

static Node *assign();

static Node *expr();

static Node *primary() {
  Token *token;
  // トークンが "(" ならば "(" expr ")" のはず
  if (consume_as_sign("(")) {
    Node *node = expr();
    expect(")");
    return node;
  }

  // ident
  token = consume_token_kind(TK_IDENT);
  if (token) {
    if (consume_as_sign("(")) {
      Node *node = new_node(ND_FUNCALL);
      node->funcname = strndup(token->str, token->len);

      node->funcargs = vector_new();
      if (!consume_as_sign(")")){
        vector_push(node->funcargs, assign());
        while (consume_as_sign(",")) {
          vector_push(node->funcargs, assign());
        }
        expect(")");
      }
      return node;
    }
    Node *node = new_node(ND_LVAR);
    LVar *lvar = find_lvar(token);
    if (lvar) {
      node->offset = lvar->offset;
    } else {
      lvar = new_lvar(token);
      node->offset = lvar->offset;
      vector_push(g_locals, lvar);
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
    return new_node_binary_ops(ND_SUB, new_node_num(0), primary());
  } else {
    return primary();
  }
}

static Node *mul() {
  Node *node = unary();

  for (;;) {
    if (consume_as_sign("*")) {
      node = new_node_binary_ops(ND_MUL, node, unary());
    } else if (consume_as_sign("/")) {
      node = new_node_binary_ops(ND_DIV, node, unary());
    } else {
      return node;
    }
  }
}

static Node *add() {
  Node *node = mul();

  for (;;) {
    if (consume_as_sign("+")) {
      node = new_node_binary_ops(ND_ADD, node, mul());
    } else if (consume_as_sign("-")) {
      node = new_node_binary_ops(ND_SUB, node, mul());
    } else {
      return node;
    }
  }
}

static Node *relational() {
  Node *node = add();

  for (;;) {
    if (consume_as_sign("<=")) {
      node = new_node_binary_ops(ND_LTE, node, add());
    } else if (consume_as_sign(">=")) {
      node = new_node_binary_ops(ND_LTE, add(), node);
    } else if (consume_as_sign("<")) {
      node = new_node_binary_ops(ND_LT, node, add());
    } else if (consume_as_sign(">")) {
      node = new_node_binary_ops(ND_LT, add(), node);
    } else {
      return node;
    }
  }
}

static Node *equality() {
  Node *node = relational();

  for (;;) {
    if (consume_as_sign("==")) {
      node = new_node_binary_ops(ND_EQ, node, relational());
    } else if (consume_as_sign("!=")) {
      node = new_node_binary_ops(ND_NE, node, relational());
    } else {
      return node;
    }
  }
}

static Node *assign() {
  Node *node = equality();

  for (;;) {
    if (consume_as_sign("=")) {
      node = new_node_binary_ops(ND_ASSIGN, node, assign());
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
  Token *token;

  // block
  if (consume_as_sign("{")) {
    node = new_node(ND_BLOCK);
    node->children = vector_new();
    while (!consume_as_sign("}")) {
      vector_push(node->children, stmt());
    }
    return node;
  }

  // if
  token = consume_token_kind(TK_IF);
  if (token) {
    node = new_node(ND_IF);
    expect("(");
    node->cond = expr();
    expect(")");
    node->then = stmt();
    token = consume_token_kind(TK_ELSE);
    if (token) {
      node->els = stmt();
    }
    return node;
  }

  // while
  token = consume_token_kind(TK_WHILE);
  if (token) {
    node = new_node(ND_WHILE);
    expect("(");
    node->cond = expr();
    expect(")");
    node->then = stmt();
    return node;
  }

  // for
  token = consume_token_kind(TK_FOR);
  if (token) {
    node = new_node(ND_FOR);
    expect("(");
    if (!consume_as_sign(";")) {
      node->init = expr();
      expect(";");
    }
    if (!consume_as_sign(";")) {
      node->cond = expr();
      expect(";");
    }
    if (!consume_as_sign(")")) {
      node->inc = expr();
      expect(")");
    }
    node->then = stmt();
    return node;
  }

  // return
  token = consume_token_kind(TK_RETURN);
  if (token) {
    node = new_node_binary_ops(ND_RETURN, expr(), NULL);
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
