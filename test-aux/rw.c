#include <stdio.h>
#include <string.h>

static const char *content = "1145141919810";

int main(int argc, char **argv)
{
	FILE *f;
	int w = 0;

	if (argc != 3 && argc != 2) {
		fprintf(stderr, "usage: %s {r|w} [filename]\n", argv[0]);
		return 1;
	}

	if (strcmp(argv[1], "w") == 0)
		w = 1;
	else if (strcmp(argv[1], "r") != 0) {
		fprintf(stderr, "usage: %s {r|w} [filename]\n", argv[0]);
		return 1;
	}

	if (argc == 2)
		if (w)
			f = stdout;
		else
			f = stdin;
	else
		f = fopen(argv[2], argv[1]);

	if (!f) {
		perror("fopen");
		return 2;
	}

	if (w) {
		if (fputs(content, f) < 0) {
			perror("fputs");
			return 3;
		}
	} else {
		char buf[256];
		if (fscanf(f, "%s", buf) != 1) {
			fprintf(stderr, "failed to get the token\n");
			return 3;
		}
		if (strcmp(buf, content) != 0) {
			fprintf(stderr, "file content is incorrect\n");
			return 4;
		}
	}

	return 0;
}
