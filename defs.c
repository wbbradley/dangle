#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_WORD (256 * 4)

void print_word(char *prior_word, char *word, int *len) {
  if (*len != 0) {
    word[*len] = '\0';
    if (strcmp(prior_word, "def") == 0 || strcmp(prior_word, "class") == 0) {
      printf("%s\n", word);
    }
    strncpy(prior_word, word, MAX_WORD);
    *len = 0;
  }
}

void append_char(int ch, char *word, int *len, char *prior_word) {
  word[*len] = ch;
  *len = (*len) + 1;
  if (*len == MAX_WORD - 1) {
    print_word(prior_word, word, len);
  }
}

int main(int argc, char *argv[]) {
  char prior_word[MAX_WORD];
  prior_word[0] = '\0';
  char word[MAX_WORD];
  int len = 0;
  while (1) {
    int ch = getc(stdin);
    if (ch == EOF) {
      print_word(prior_word, word, &len);
      return EXIT_SUCCESS;
    }

    if (ch == '#') {
      print_word(prior_word, word, &len);
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
      append_char(ch, word, &len, prior_word);
    } else if (len != 0 && isalnum(ch)) {
      append_char(ch, word, &len, prior_word);
    } else {
      print_word(prior_word, word, &len);
    }
  }

  return EXIT_SUCCESS;
}
