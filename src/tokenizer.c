#include "pcc.h"

#include "stdbool.h"

// トークナイズ状況
typedef struct TokenizerContext {
  // トークナイズ対象の文字列の先頭ポインタ
  char *p;
  // トークン連結リストの最後尾ポインタ
  Token *cur;
} TokenizerContext;

static const char *const SIGNES[] = {
  ">=", "<=",
  ">", "<", "(", ")",
  "+", "-", "*", "/",
  ";", "="
};

static bool is_alnum(char c) {
  return
    ('a' <= c && c <= 'z') ||
    ('A' <= c && c <= 'Z') ||
    (c == '_');
}

static const char* token_kind_str(TokenKind kind) {
  switch (kind) {
#define XX(name) case name: return #name;
    TOKEN_KIND_MAP(XX)
#undef XX
    default: return "<unknown>";
  }
}

// 新しいトークンを作成して cur に繋げる
static Token *new_token(
  TokenKind kind,
  Token* cur,
  char* str,
  const int len
) {
  assert(cur != NULL);
  assert(str != NULL);

  Token *tok = calloc(1, sizeof(Token));
  tok->kind = kind;
  tok->str = str;
  tok->len = len;
  cur->next = tok;

  DEBUGF("tokenize \"%.*s\" as %s.\n", len, str, token_kind_str(kind));
  return tok;
}

/**
 * @param ctx [IN/OUT] トークナイズ状況
 * @return 記号としてトークナイズされたかどうか
 */
static bool consume_as_sign(
  TokenizerContext *ctx
) {
  assert(ctx != NULL);
  assert(ctx->p != NULL);
  assert(ctx->cur != NULL);

  for (size_t i = 0; i < sizeof(SIGNES) / sizeof(SIGNES[0]); ++i) {
    const size_t len = strlen(SIGNES[i]);
    if (strncmp(ctx->p, SIGNES[i], len) == 0) {
      ctx->cur = new_token(TK_SIGN, ctx->cur, ctx->p, len);
      ctx->p += len;
      return true;
    }
  }
  return false;
}

/**
 * @param ctx [IN/OUT] トークナイズ状況
 * @return 識別子としてトークナイズされたかどうか
 */
static bool consume_as_ident(
  TokenizerContext *ctx
) {
  assert(ctx != NULL);
  assert(ctx->p != NULL);
  assert(ctx->cur != NULL);

  char *str = ctx->p;
  size_t len = 0;
  while (*ctx->p) {
    if (is_alnum(*ctx->p)) {
      len += 1;
      ctx->p += 1;
    } else {
      break;
    }
  }
  if (len <= 0) {
    return false;
  }

  ctx->cur = new_token(TK_IDENT, ctx->cur, str, len);
  return true;
}

/**
 * @param ctx [IN/OUT] トークナイズ状況
 * @return 数値としてトークナイズされたかどうか
 */
static bool consume_as_digit(
  TokenizerContext *ctx
) {
  if (!isdigit(*ctx->p)) {
    return false;
  }
  char* head = ctx->p;
  int val = strtol(ctx->p, &ctx->p, 10);
  ctx->cur = new_token(TK_NUM, ctx->cur, head, ctx->p - head);
  ctx->cur->val = val;
  return true;
}

/**
 * @param ctx [IN/OUT] トークナイズ状況
 * @param str [IN] 同じかどうか判定する単語
 * @param kind [IN] トークナイズする際のトークン種別
 * @return 予約語としてトークナイズされたかどうか
 */
static bool consume_as_reserved(
  TokenizerContext *ctx,
  char* str,
  const TokenKind kind
) {
  assert(ctx != NULL);
  assert(ctx->p != NULL);
  assert(ctx->cur != NULL);

  const size_t len = strlen(str);
  if (strncmp(ctx->p, str, len) == 0 && !is_alnum((ctx->p)[len])) {
    ctx->cur = new_token(kind, ctx->cur, ctx->p, len);
    ctx->p += len;
    return true;
  }
  return false;
}

Token *tokenize(char* p) {
  assert(p != NULL);

  Token head;
  head.next = NULL;
  TokenizerContext ctx = {.p = p, &head};

  while (*ctx.p) {
    // 空白文字をスキップ
    if (isspace(*ctx.p)) {
      ctx.p++;
      continue;
    }
    if (consume_as_reserved(&ctx, "return", TK_RETURN)) {
      continue;
    }
    if (consume_as_reserved(&ctx, "if", TK_IF)) {
      continue;
    }
    if (consume_as_reserved(&ctx, "else", TK_ELSE)) {
      continue;
    }
    if (consume_as_reserved(&ctx, "while", TK_WHILE)) {
      continue;
    }
    if (consume_as_reserved(&ctx, "for", TK_FOR)) {
      continue;
    }
    // 予約語
    if (consume_as_sign(&ctx)) {
      continue;
    } 
    // 数値
    if (consume_as_digit(&ctx)) {
      continue;
    }
    // 識別子
    if (consume_as_ident(&ctx)) {
      continue;
    }
    // 上記以外
    error_at(ctx.p, "トークナイズできません");
  }

  new_token(TK_EOF, ctx.cur, ctx.p, 0);
  return head.next;
}
