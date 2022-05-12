#undef NDEBUG
#include <assert.h>
#include <stdio.h>

int main()
{
	int a, b;
	while (scanf("%d%d", &a, &b) == 2) {
		assert(a + b == 3);
		printf("%d\n", a + b);
	}
	return 0;
}
