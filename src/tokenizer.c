#include "9cc.h"

// 新しいトークンを作成して cur に繋げる
static Token *new_token(TokenKind kind, Token *cur, char *str, int len) {
  Token *tok = calloc(1, sizeof(Token));
  tok->kind = kind;
  tok->str = str;
  tok->len = len;
  cur->next = tok;
  return tok;
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

    if (strncmp(p, ">=", 2) == 0 || strncmp(p, "<=", 2) == 0) {
      cur = new_token(TK_RESERVED, cur, p, 2);
      p += 2;
      continue;
    }

    if (
      *p == '>' || *p == '<' || *p == '(' || *p == ')' ||
      *p == '+' || *p == '-' || *p == '*' || *p == '/'
    ) {
      cur = new_token(TK_RESERVED, cur, p, 1);
      p += 1;
      continue;
    }

    if (isdigit(*p)) {
      cur = new_token(TK_NUM, cur, p, 0);
      cur->val = strtol(p, &p, 10);
      continue;
    }

    error_at(p, "トークナイズできません");
  }

  new_token(TK_EOF, cur, p, 0);
  return head.next;
}
