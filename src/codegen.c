#include "pcc.h"

#include "error.h"

#define p(...) do {printf(__VA_ARGS__);printf("\n");} while(0);
#define emit(...) do {printf("\t");p(__VA_ARGS__)} while(0);

static unsigned long g_label_count = 0;
static const char *argregs[] = {"rdi", "rsi", "rdx", "rcx", "r8", "r9"};
static const size_t MAX_ARGS = sizeof(argregs) / sizeof(argregs[0]);

static unsigned long generate_label_id() {
  return ++g_label_count;
}

static void gen_lval(Node *node) {
  assert(node != NULL);
  assert(node->kind == ND_LVAR);

  emit("mov rax, rbp");
  emit("sub rax, %d", node->offset);
  emit("push rax");
}

static void gen(Node *node) {
  assert(node != NULL);
  DEBUGF("consume node %s.\n", node_kind_str(node->kind));

  // block
  if (node->kind == ND_BLOCK) {
    assert(node->children != NULL);
    for (int i = 0; i < node->children->length; i += 1) {
      gen((Node*)vector_at(node->children, i));
      // 式の評価結果としてスタックに一つの値が残っている
      // はずなので、スタックが溢れないようにポップしておく
      emit("pop rax");
    }
    return;
  }

  // control syntax
  if (node->kind == ND_IF) {
    const unsigned long label_id = generate_label_id();
    assert(node->cond != NULL);
    assert(node->then != NULL);
    gen(node->cond);
    emit("pop rax");
    emit("cmp rax, 0");
    if (node->els) {
      emit("je  .Lelse%ld", label_id);
      gen(node->then);
      emit("jmp .Lend%ld", label_id);
      p(".Lelse%ld:", label_id);
      gen(node->els);
    } else {
      emit("je  .Lend%ld", label_id);
      gen(node->then);
    }
    p(".Lend%ld:", label_id);
    return;
  }
  if (node->kind == ND_WHILE) {
    const unsigned long label_id = generate_label_id();
    assert(node->cond != NULL);
    assert(node->then != NULL);
    p(".Lbegin%ld:", label_id);
    gen(node->cond);
    emit("pop rax");
    emit("cmp rax, 0");
    emit("je  .Lend%ld", label_id);
    gen(node->then);
    emit("jmp .Lbegin%ld", label_id);
    p(".Lend%ld:", label_id);
    return;
  }
  if (node->kind == ND_FOR) {
    const unsigned long label_id = generate_label_id();
    assert(node->then != NULL);
    if (node->init) {
      gen(node->init);
    }
    p(".Lbegin%ld:", label_id);
    if (node->cond) {
      gen(node->cond);
      emit("pop rax");
      emit("cmp rax, 0");
      emit("je  .Lend%ld", label_id);
    }
    gen(node->then);
    if (node->inc) {
      gen(node->inc);
    }
    emit("jmp .Lbegin%ld", label_id);
    p(".Lend%ld:", label_id);
    return;
  }
  if (node->kind == ND_RETURN) {
    assert(node->lhs != NULL);
    gen(node->lhs);
    emit("pop rax");
    emit("mov rsp, rbp");
    emit("pop rbp");
    emit("ret");
    return;
  }

  // number, variable or assignment
  if (node->kind == ND_NUM) {
    emit("push %d", node->val);
    return;
  }
  if (node->kind == ND_LVAR) {
    gen_lval(node);
    emit("pop rax");
    emit("mov rax, [rax]");
    emit("push rax");
    return;
  }
  if (node->kind == ND_ASSIGN) {
    assert(node->lhs != NULL);
    gen_lval(node->lhs);
    assert(node->rhs != NULL);
    gen(node->rhs);

    emit("pop rdi");
    emit("pop rax");
    emit("mov [rax], rdi");
    emit("push rdi");
    return;
  }

  // function call
  if (node->kind == ND_FUNCALL) {
    assert(node->funcname != NULL);
    assert(node->funcargs != NULL);

    int arg_count = 0;
    for (; arg_count < node->funcargs->length; arg_count += 1) {
      Node *arg = vector_at(node->funcargs, arg_count);
      gen(arg);
    }
    // 6 個までの引数しか対応していないため
    assert(arg_count <= MAX_ARGS);
    for (int i = arg_count - 1; i >= 0; i -= 1) {
      emit("pop %s", argregs[i]);
    }

    /**
     * @see https://github.com/rui314/chibicc/commit/ee42303
     * 関数呼出の前に RSP の値が 16 の倍数でなければならないためそのための対応をする
     */
    const unsigned long label_id = generate_label_id();
    emit("mov rax, rsp");
    emit("and rax, 15");
    emit("jnz .L.call.%ld", label_id);
    emit("mov rax, 0");
    emit("call %s", node->funcname);
    emit("jmp .L.end.%ld", label_id);
    p(".L.call.%ld:", label_id);
    emit("sub rsp, 8");
    emit("mov rax, 0");
    emit("call %s", node->funcname);
    emit("add rsp, 8");
    p(".L.end.%ld:", label_id);
    emit("push rax");
    return;
  }

  // binary operators
  assert(node->lhs != NULL);
  gen(node->lhs);
  assert(node->rhs != NULL);
  gen(node->rhs);

  emit("pop rdi");
  emit("pop rax");

  switch(node->kind) {
  case ND_EQ:
    emit("cmp rax, rdi");
    emit("sete al");
    emit("movzb rax, al");
    break;
  case ND_NE:
    emit("cmp rax, rdi");
    emit("setne al");
    emit("movzb rax, al");
    break;
  case ND_LTE:
    emit("cmp rax, rdi");
    emit("setle al");
    emit("movzb rax, al");
    break;
  case ND_LT:
    emit("cmp rax, rdi");
    emit("setl al");
    emit("movzb rax, al");
    break;
  case ND_ADD:
    emit("add rax, rdi");
    break;
  case ND_SUB:
    emit("sub rax, rdi");
    break;
  case ND_MUL:
    emit("imul rax, rdi");
    break;
  case ND_DIV:
    emit("cqo");
    emit("idiv rdi");
    break;
  default:
    fprintf(stderr, "unknown node kind: %d", node->kind);
    fflush(stderr);
    assert(0);
  }
  emit("push rax");
}

/**
 * @param functions Vector<Function>
 */
void codegen(Vector *functions) {
  assert(functions != NULL);
  // アセンブリの前半部分を出力
  p(".intel_syntax noprefix");

  for (int fi = 0; fi < functions->length; fi += 1) {
    Function *fn = vector_at(functions, fi);
    assert(fn != NULL);
    assert(fn->name != NULL);
    assert(fn->body != NULL);
    assert(fn->params != NULL);
    assert(fn->locals != NULL);

    // プロローグ
    DEBUGF("codegen of %s()\n", fn->name);
    p(".global %s", fn->name);
    p("%s:", fn->name);

    // 変数分の領域を確保する
    const int localsSize = vector_empty(fn->locals) ? 0 : ((LVar*)vector_last(fn->locals))->offset + 8;
    DEBUGF("local variables: %d bytes\n", localsSize);
    emit("push rbp");
    emit("mov rbp, rsp");
    emit("sub rsp, %d", localsSize);

    // 引数をスタックに展開
    assert(fn->params->length <= MAX_ARGS);
    for (int pi = 0; pi < fn->params->length; pi += 1) {
      LVar *param = vector_at(fn->params, pi);
      emit("mov [rbp-%d], %s", param->offset, argregs[pi]);
    }

    // コード生成
    for (int bi = 0; bi < fn->body->length; bi += 1) {
      Node *node = vector_at(fn->body, bi);
      gen(node);
    }

    // 式の評価結果としてスタックに一つの値が残っている
    // はずなので、スタックが溢れないようにポップしておく
    emit("pop rax");

    // エピローグ
    // 最後の式の結果が rax に残っているのでそれが戻り値になる
    emit("mov rsp, rbp");
    emit("pop rbp");
    emit("ret");
  }
}
