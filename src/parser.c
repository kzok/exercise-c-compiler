#include "pcc.h"

#include <stdbool.h>
#include <string.h>

// ローカル変数
static Vector *g_locals = NULL; // Vector<LVar>

static void seek_token() {
  assert(g_tokens != NULL);
  g_tokens = g_tokens->next;
}

// 次のトークンが期待している記号のときには、トークンを一つ読み進める。
// それ以外の場合にはエラーを報告する。
static void expect(char *op) {
  assert(op != NULL);
  assert(g_tokens != NULL);

  if (
    g_tokens->kind != TK_SIGN
    || g_tokens->len != strlen(op)
    || memcmp(g_tokens->str, op, g_tokens->len)
  ) {
    error_at(g_tokens->str, "'%s' ではありません", op);
  }
  seek_token();
}

// 次のトークンが数値の場合、トークンを一つ読み進めてその数値を返す。
// それ以外の場合にはエラーを報告する。
static int expect_number() {
  assert(g_tokens != NULL);

  if (g_tokens->kind != TK_NUM) {
    error_at(g_tokens->str, "数ではありません");
  }
  int val = g_tokens->val;
  seek_token();
  return val;
}

/**
 * @return ident string
 */
static char* expect_ident() {
  assert(g_tokens != NULL);
  if (g_tokens->kind != TK_IDENT) {
    error_at(g_tokens->str, "識別子ではありません");
  }
  char *ident = strndup(g_tokens->str, g_tokens->len);
  seek_token();
  return ident;
}

static bool at_eof() {
  assert(g_tokens != NULL);
  return g_tokens->kind == TK_EOF;
}

// 次のトークンが期待している記号のときには、トークンを一つ読み進めて真を返す。
// それ以外の場合には偽を返す。
static bool consume_as_sign(char *op) {
  assert(op != NULL);

  if (
      g_tokens->kind != TK_SIGN ||
      g_tokens->len != strlen(op) ||
      memcmp(g_tokens->str, op, g_tokens->len)
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
  assert(g_tokens != NULL);

  if (g_tokens->kind != kind) {
    return NULL;
  }
  Token *current = g_tokens;
  seek_token();
  return current;
}

static Node *new_node(NodeKind kind) {
  DEBUGF("new node: %s\n", node_kind_str(kind));
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
    size_t len = strlen(var->name);
    if (len == tok->len && !memcmp(tok->str, var->name, len)) {
      return var;
    }
  }
  return NULL;
}

static LVar *new_lvar(char *str) {
  assert(g_locals != NULL);
  LVar *lvar = calloc(1, sizeof(LVar));
  lvar->name = str;
  // 今は int 型しかないので 8 バイト固定
  lvar->offset = vector_empty(g_locals) ? 8 : ((LVar*)vector_last(g_locals))->offset + 8;
  vector_push(g_locals, lvar);
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
      lvar = new_lvar(strndup(token->str, token->len));
      node->offset = lvar->offset;
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

/**
 * @return Vector<LVar>
 */
static Vector *function_params() {
  Vector *params = vector_new();

  expect("(");
  if (consume_as_sign(")")){
    return params;
  }
  vector_push(params, new_lvar(expect_ident()));
  while (consume_as_sign(",")) {
    vector_push(params, new_lvar(expect_ident()));
  }
  expect(")");

  return params;
}

static Function *function() {
  assert(g_locals == NULL);
  g_locals = vector_new();

  Function *fn = calloc(1, sizeof(Function));
  fn->name = expect_ident();
  fn->body = vector_new();

  fn->params = function_params();

  expect("{");
  while (!consume_as_sign("}")) {
    vector_push(fn->body, stmt());
  }
  fn->locals = g_locals;
  g_locals = NULL;

  return fn;
}

/**
 * @returns Vector<Function>
 */
Vector *program() {
  Vector *functions = vector_new();

  while (!at_eof()) {
    vector_push(functions, function()); 
  }

  return functions;
}
