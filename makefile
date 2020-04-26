CFLAGS=-std=c11 -g -static -Wall
TARGET=9cc
SRCS=$(wildcard src/*.c)
OBJS=$(SRCS:.c=.o)

# Commands

.PHONY: test
test: $(TARGET)
	./test.sh

.PHONY: test_debug
test_debug: build_debug
	./test.sh

.PHONY: build_debug
build_debug: clean
build_debug: CFLAGS += -DDEBUG
build_debug: $(TARGET)

.PHONY: clean
clean:
	rm -f $(TARGET) $(OBJS) *~ tmp*

# Dependencies

$(TARGET): $(OBJS)
	gcc -o $(TARGET) $(OBJS) $(LDFLAGS)

$(OBJS): src/9cc.h
