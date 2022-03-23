#include <stdlib.h>

int main()
{
	char *x = malloc(256 << 20);
	if (!x)
		/* The malloc function may return NULL if out-of-memory.  Return
		   a non-zero value to correctly report allocation failure. */
		return 1;

	memset(x, 1, 256 << 20);

	/* Prevent optimization. */
	__asm__ volatile ("":::"memory");
	return 0;
}
