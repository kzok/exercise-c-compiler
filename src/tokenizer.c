#include "9cc.h"

#include "stdbool.h"

static const char *const RESERVED[] = {
  ">=", "<=",
  ">", "<", "(", ")",
  "+", "-", "*", "/",
  ";", "="
};

// 新しいトークンを作成して cur に繋げる
static Token *new_token(TokenKind kind, Token *cur, char *str, int len) {
  Token *tok = calloc(1, sizeof(Token));
  tok->kind = kind;
  tok->str = str;
  tok->len = len;
  cur->next = tok;
  return tok;
}

/**
 * @param p [IN/OUT] トークナイズ対象の文字列の先頭ポインタ
 * @param cur [IN/OUT] トークンカーソルの先頭ポインタ
 * @return 予約語としてトークナイズされたかどうか
 */
static bool tokenize_if_reserved(
  char **p,
  Token **cur
) {
  for (size_t i = 0; i < sizeof(RESERVED) / sizeof(RESERVED[0]); ++i) {
    const size_t len = strlen(RESERVED[i]);
    if (strncmp(*p, RESERVED[i], len) == 0) {
      *cur = new_token(TK_RESERVED, *cur, *p, len);
      *p += len;
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
static bool tokenize_if_ident(
  char **p,
  Token **cur
) {
  char *head = *p;
  size_t len = 0;
  while (**p) {
    if ('a' <= **p && **p <= 'z') {
      len += 1;
      *p += 1;
    } else {
      break;
    }
  }
  if (len <= 0) {
    return false;
  }

  *cur = new_token(TK_IDENT, *cur, head, len);
  return true;
}

Token *tokenize(char *p) {
  Token head;
  head.next = NULL;
  Token *cur = &head;

  while (*p) {
    // 空白文字をスキップ
    if (isspace(*p)) {
      p++;
      continue;
    }

    if (tokenize_if_reserved(&p, &cur)) {
      continue;
    } 

    if (isdigit(*p)) {
      cur = new_token(TK_NUM, cur, p, 0);
      cur->val = strtol(p, &p, 10);
      continue;
    }

    if (tokenize_if_ident(&p, &cur)) {
      continue;
    }

    error_at(p, "トークナイズできません");
  }

  new_token(TK_EOF, cur, p, 0);
  return head.next;
}
