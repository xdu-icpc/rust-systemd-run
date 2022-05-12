#include <stdio.h>
#include </dev/random>

int main()
{
	int a, b;
	while (scanf("%d%d", &a, &b) == 2)
		printf("%d\n", a + b);
	return 0;
}
