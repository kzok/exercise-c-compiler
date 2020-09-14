#include "pcc.h"

#include "error.h"

#define p(...) do {printf(__VA_ARGS__);printf("\n");} while(0);
#define emit(...) do {printf("\t");p(__VA_ARGS__)} while(0);

static unsigned long g_label_count = 0;

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

void gen(Node *node) {
  assert(node != NULL);

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
