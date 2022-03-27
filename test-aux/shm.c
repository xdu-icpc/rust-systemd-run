#include <sys/shm.h>
#include <stdio.h>

int main()
{
	int r = shmget(114, 4096, IPC_CREAT | IPC_EXCL);
	if (r < 0) {
		perror("shmget");
		return 1;
	}
	return 0;
}
