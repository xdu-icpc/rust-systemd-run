#include <atomic>
#include <ctime>
#include <thread>

std::atomic<bool> failed(false);

void thread()
{
	struct timespec sp;
	do {
		for (volatile int i = 0; i < 10000; i++);
		if (clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &sp) != 0) {
			failed.store(true);
			break;
		}
	} while (sp.tv_sec < 1);
}

int main()
{
	std::thread a(thread);
	std::thread b(thread);
	a.join();
	b.join();
	return failed;
}
