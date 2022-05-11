#include <unistd.h>
#include <stdio.h>

int main()
{
	int fd[2];
	for (int i = 0; i < 114; i++)
		if (pipe(fd) < 0) {
			perror("pipe");
			return 1;
		}
	return 0;
}
