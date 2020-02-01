CFLAGS=-std=c11 -g -static -Wall
TARGET=9cc
SRCS=$(wildcard src/*.c)
OBJS=$(SRCS:.c=.o)

9cc: $(OBJS)
	gcc -o $(TARGET) $(OBJS) $(LDFLAGS)

$(OBJS): src/9cc.h

test: $(TARGET)
	./test.sh

clean:
	rm -f $(TARGET) $(OBJS) *~ tmp*

.PHONY: test clean
