#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_WORD (256 * 4)

void print_word(char *word, int *len) {
  if (*len != 0) {
    word[*len] = '\0';
    printf("%s\n", word);
    *len = 0;
  }
}

void append_char(int ch, char *word, int *len) {
  word[*len] = ch;
  *len = (*len) + 1;
  if (*len == MAX_WORD - 1) {
    print_word(word, len);
  }
}

void read_until_quotes(int quote, int count) {
  int left = count;
  int ch;
  while ((ch = getc(stdin)) != EOF) {
    if (ch != quote) {
      left = count;
      continue;
    } else if (ch == quote) {
      left -= 1;
      if (left == 0) {
        return;
      }
    }
  }
  ungetc(EOF, stdin);
}

int main(int argc, char *argv[]) {
  char word[MAX_WORD];
  int len = 0;
  while (1) {
    int ch = getc(stdin);

  l_just_chomped:
    if (ch == EOF) {
      print_word(word, &len);
      return EXIT_SUCCESS;
    }

    if (ch == '#') {
      print_word(word, &len);
      do {
        ch = getc(stdin);
        if (ch == EOF) {
          return EXIT_SUCCESS;
        }
        if (ch == '\n') {
          break;
        }
      } while (1);
    }

    if (isalpha(ch) || ch == '_') {
      append_char(ch, word, &len);
    } else if (len != 0 && isalnum(ch)) {
      append_char(ch, word, &len);
    } else {
      print_word(word, &len);
    }
  }

  return EXIT_SUCCESS;
}
