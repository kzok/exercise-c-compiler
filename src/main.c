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

  g_user_input = argv[1];
  g_tokens = tokenize(g_user_input);
  codegen(program());
  return 0;
}
