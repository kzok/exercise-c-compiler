#include "9cc.h"

#include "stdbool.h"

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

// 新しいトークンを作成して cur に繋げる
static Token *new_token(
  TokenKind kind,
  Token* cur,
  char* str,
  const int len
) {
  Token *tok = calloc(1, sizeof(Token));
  tok->kind = kind;
  tok->str = str;
  tok->len = len;
  cur->next = tok;
  DEBUGF("[debug] tokenize \"%.*s\" as kind:%d.\n", len, str, kind);
  return tok;
}

/**
 * @param p [IN/OUT] トークナイズ対象の文字列の先頭ポインタ
 * @param cur [IN/OUT] トークンカーソルの先頭ポインタ
 * @return 記号としてトークナイズされたかどうか
 */
static bool consume_as_sign(
  char** pp,
  Token** cur
) {
  for (size_t i = 0; i < sizeof(SIGNES) / sizeof(SIGNES[0]); ++i) {
    const size_t len = strlen(SIGNES[i]);
    if (strncmp(*pp, SIGNES[i], len) == 0) {
      *cur = new_token(TK_SIGN, *cur, *pp, len);
      *pp += len;
      return true;
    }
  }
  return false;
}

/**
 * @param p [IN/OUT] トークナイズ対象の文字列の先頭ポインタ
 * @param cur [IN/OUT] トークンカーソルの先頭ポインタ
 * @return 識別子としてトークナイズされたかどうか
 */
static bool consume_as_ident(
  char** pp,
  Token** cur
) {
  char *str = *pp;
  size_t len = 0;
  while (**pp) {
    if (is_alnum(**pp)) {
      len += 1;
      *pp += 1;
    } else {
      break;
    }
  }
  if (len <= 0) {
    return false;
  }

  *cur = new_token(TK_IDENT, *cur, str, len);
  return true;
}

/**
 * @param p [IN/OUT] トークナイズ対象の文字列の先頭ポインタ
 * @param cur [IN/OUT] トークンカーソルの先頭ポインタ
 * @param str [IN] 同じかどうか判定する単語
 * @param kind [IN] トークナイズする際のトークン種別
 * @return 予約語としてトークナイズされたかどうか
 */
static bool consume_as_reserved(
  char** pp,
  Token** cur,
  const char* str,
  const TokenKind kind
) {
  const size_t len = strlen(str);
  if (strncmp(*pp, str, len) == 0 && !is_alnum((*pp)[len])) {
    *cur = new_token(TK_RETURN, *cur, *pp, len);
    *pp += len;
    return true;
  }
  return false;
}

Token *tokenize(char* p) {
  Token head;
  head.next = NULL;
  Token *cur = &head;

  while (*p) {
    // 空白文字をスキップ
    if (isspace(*p)) {
      p++;
      continue;
    }
    // リターン文
    if (consume_as_reserved(&p, &cur, "return", TK_RETURN)) {
      continue;
    }
    // 予約語
    if (consume_as_sign(&p, &cur)) {
      continue;
    } 
    // 数値
    if (isdigit(*p)) {
      char* head = p;
      int val = strtol(p, &p, 10);
      cur = new_token(TK_NUM, cur, head, p - head);
      cur->val = val;
      continue;
    }
    // 識別子
    if (consume_as_ident(&p, &cur)) {
      continue;
    }
    // 上記以外
    error_at(p, "トークナイズできません");
  }

  new_token(TK_EOF, cur, p, 0);
  return head.next;
}
