/* Ensure that the SGID bit is not in-effect. */
#include <unistd.h>

int main()
{
	return getgid() != getegid();
}
