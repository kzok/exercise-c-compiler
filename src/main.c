#define ENTRYPOINT

#include "9cc.h"

#include <string.h>

// 新しいトークンを作成して cur に繋げる
Token *new_token(TokenKind kind, Token *cur, char *str, int len) {
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

/**
 * ENTRY POINT
 */

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "引数の個数が正しくありません\n");
    return 1;
  }

  // トークナイズしてパースする
  g_user_input = argv[1];
  g_token = tokenize(g_user_input);
  Node *node = expr();

  // アセンブリの前半部分を出力
  printf(".intel_syntax noprefix\n");
  printf(".global main\n");
  printf("main:\n");

  // 抽象構文木を下りながらコード生成
  gen(node);

  // スタックトップに式全体の値が残っているはずなので
  // それを rax にロードして関数からの戻り値とする
  printf("  pop rax\n");
  printf("  ret\n");
  return 0;
}
