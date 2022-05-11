#include <atomic>
#include <thread>

void thread(int x)
{
	if (x == 0)
		return;
	auto t = std::thread(thread, x - 1);
	t.join();
}

int main()
{
	thread(100);
	return 0;
}
