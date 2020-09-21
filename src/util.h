#ifndef _UTIL_H_INCLUDED
#define _UTIL_H_INCLUDED

#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
#include <assert.h>

// Vector

typedef struct {
  void **data;
  size_t capacity;
  size_t length;
} Vector;

static inline Vector *vector_new() {
  Vector *vector = (Vector*)calloc(sizeof(Vector), 1);
  vector->capacity = 16;
  vector->data = (void**)calloc(sizeof(void *), vector->capacity);
  vector->length = 0;
  return vector;
}

static inline void vector_push(Vector *vector, void* el) {
  assert(vector != NULL);
  if (vector->length == vector->capacity) {
    vector->capacity *= 2;
    vector->data = (void**)realloc(vector->data, sizeof(void *) * vector->capacity);
    // zero fill to additional memory spaces
    memset(vector->data + vector->length, 0, vector->capacity - vector->length);
  }
  vector->data[vector->length] = el;
  vector->length += 1;
}

static inline void* vector_at(Vector *vector, size_t index) {
  assert(vector != NULL);
  assert(index < vector->length);
  return vector->data[index];
}

static inline bool vector_empty(Vector *vector) {
  assert(vector != NULL);
  return vector->length == 0;
}

static inline void* vector_last(Vector *vector) {
  assert(vector != NULL);
  assert(vector->length > 0);
  return vector->data[vector->length - 1];
}

#endif // #ifndef _UTIL_H_INCLUDED 
