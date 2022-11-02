#include <signal.h>
#include <stdlib.h>
#include <unistd.h>

int main()
{
	struct sigaction sa = {
		.sa_handler = SIG_IGN,
	};
	sigaction(SIGTERM, &sa, NULL);
	while (1)
		usleep(10000);
}
