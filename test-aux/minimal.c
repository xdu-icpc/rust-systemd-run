/* A minimal runable program.  */

_Noreturn void _Exit(int);

_Noreturn void _start()
{
#if defined(__GNUC__) && defined(__linux__) && defined(__x86_64__)
	__asm__ volatile (
		"mov $60, %rax\n"
		"mov $0, %rdi\n"
		"syscall\n"
	);
#elif defined(__GNUC__) && defined(__linux__) && defined(__i386__)
	__asm__ volatile (
		"mov $1, %eax\n"
		"mov $0, %ebx\n"
		"int $0x80\n"
	);
#else
	/* Use libc function when we don't know how to exit.  --as-needed -lc
	   should be used linking this program.  */
	_Exit(0);
#endif

	/* Suppress a warning.  */
#ifdef __GNUC__
	__builtin_unreachable();
#endif
}
