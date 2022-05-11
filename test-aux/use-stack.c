#include <string.h>

#include "barrier.h"

int main()
{
	char x[200 << 20];
	memset(x, 0, sizeof(x));
	barrier();
	return 0;
}
