#include <errno.h>
#include <stdio.h>
#include <unistd.h>

int main()
{
	/* UID 514 should not exist in a separate user namespace. */
	return setuid(514) >= 0 || errno != EINVAL;
}
