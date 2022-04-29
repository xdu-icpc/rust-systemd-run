#[cfg(feature = "unified_cgroup")]
mod memory_tests_need_unified_cgroup {
    use byte_unit::Byte;
    use systemd_run::RunUser;
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/memory");

    #[async_std::test]
    async fn test_memory_ok() {
        let r = RunUser::new(PATH)
            .memory_max(Byte::from_str("384 MB").unwrap())
            .memory_swap_max(Byte::from(0usize))
            .start()
            .await
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(
            !r.is_failed(),
            "allocating 256 MB should be fine with MemoryMax=384MB"
        );
    }

    #[async_std::test]
    #[cfg(feature = "systemd_236")]
    async fn test_memory_limit_exceed() {
        let r = RunUser::new(PATH)
            .memory_max(Byte::from_str("128 MB").unwrap())
            .memory_swap_max(Byte::from(0usize))
            .collect_on_fail()
            .start()
            .await
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(
            r.is_failed(),
            "allocating 256 MB should fail with MemoryMax=128MB"
        );
    }
}
