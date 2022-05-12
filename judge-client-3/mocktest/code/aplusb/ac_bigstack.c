#include <stdio.h>
#include <string.h>

int main()
{
	char x[32 << 20];
	memset(x, 1, sizeof(x));
	__asm__ volatile ("" ::: "memory");

	int a, b;
	while (scanf("%d%d", &a, &b) == 2)
		printf("%d\n", a + b);
	return 0;
}
