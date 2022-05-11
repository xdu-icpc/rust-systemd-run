#ifndef _RUST_SYSTEMD_RUN_TEST_AUX_BARRIER_H
#define _RUST_SYSTEMD_RUN_TEST_AUX_BARRIER_H

#ifdef __GNUC__
#define barrier() __asm__ volatile ("":::"memory")
#else
#warning "don't know how to prevent optimization, result may be incorrect"
#define barrier() ((void)0)
#endif

#endif
