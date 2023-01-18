#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_WORD (256 * 4)

#define SPACE 0
#define VALUE 1
#define PUNCT 2

struct word {
  char word[MAX_WORD];
  int column;
};

void print_word(struct word *prior_word, struct word *word, int *len) {
  if (*len != 0) {
    if (strcmp(prior_word->word, "def") == 0 ||
        strcmp(prior_word->word, "class") == 0) {
      printf("%s\n", word->word);
    } else if (prior_word->column == 0 && strcmp(word->word, "=") == 0) {
      printf("%s\n", prior_word->word);
    }
    strncpy(prior_word->word, word->word, MAX_WORD);
    prior_word->column = word->column;
    *len = 0;
  }
}

void append_char(int column, int ch, struct word *word, int *len,
                 struct word *prior_word) {
  word->word[*len] = ch;
  if (*len == 0) {
    word->column = column;
  }
  *len = (*len) + 1;
  word->word[*len] = '\0';
  // printf("word = %s\n", word);
  if (*len == MAX_WORD - 1) {
    print_word(prior_word, word, len);
  }
}

int ctype(int ch) {
  if (isspace(ch)) {
    return SPACE;
  } else if (isalnum(ch) || ch == '_') {
    return VALUE;
  } else if (ispunct(ch)) {
    return PUNCT;
  } else {
    return SPACE;
  }
}

int main(int argc, char *argv[]) {
  struct word prior_word, word;
  prior_word.word[0] = '\0';
  prior_word.column = 0;

  int len = 0;
  int last_type = SPACE;
  int column = 0;
  while (1) {
    int ch = getc(stdin);
    if (ch == EOF) {
      print_word(&prior_word, &word, &len);
      return EXIT_SUCCESS;
    }

    if (ch == '#') {
      print_word(&prior_word, &word, &len);
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

    int type = ctype(ch);
    // printf("type(%c) = %d\nlast_type = %d\n", ch, type, last_type);
    if (type != last_type) {
      if (last_type != SPACE) {
        print_word(&prior_word, &word, &len);
      }
      if (type != SPACE) {
        append_char(column, ch, &word, &len, &prior_word);
        last_type = type;
      }
      last_type = type;
    } else {
      if (type != SPACE) {
        // printf("appending %c...\n", ch);
        append_char(column, ch, &word, &len, &prior_word);
      }
    }

    if (ch != '\n') {
      column += 1;
    } else {
      column = 0;
    }
  }

  return EXIT_SUCCESS;
}
