#define ENTRYPOINT

#include "pcc.h"

/**
 * ENTRY POINT
 */

int main(int argc, char **argv) {
  DEBUGF("==================== START PROCESS ==================== \n");

  if (argc != 2) {
    fprintf(stderr, "引数の個数が正しくありません\n");
    return 1;
  }

  // トークナイズしてパースする
  g_user_input = argv[1];
  g_token = tokenize(g_user_input);
  program();

  // アセンブリの前半部分を出力
  printf(".intel_syntax noprefix\n");
  printf(".global main\n");
  printf("main:\n");

  // プロローグ
  // 変数分の領域を確保する
  const int localsSize = g_locals == NULL ? 0 : g_locals->offset + 8;
  DEBUGF("total size of local variables: %d bytes\n", localsSize);
  printf("  push rbp\n");
  printf("  mov rbp, rsp\n");
  printf("  sub rsp, %d\n", localsSize);

  // 先頭の式から順にコード生成
  for (int i = 0; g_code[i]; i++) {
    gen(g_code[i]);

    // 式の評価結果としてスタックに一つの値が残っている
    // はずなので、スタックが溢れないようにポップしておく
    printf("  pop rax\n");
  }

  // エピローグ
  // 最後の式の結果が rax に残っているのでそれが戻り値になる
  printf("  mov rsp, rbp\n");
  printf("  pop rbp\n");
  printf("  ret\n");
  return 0;
}
