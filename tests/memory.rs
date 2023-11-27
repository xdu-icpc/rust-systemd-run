#[cfg(feature = "unified_cgroup")]
mod memory_tests_need_unified_cgroup {
    use byte_unit::Byte;
    use byte_unit::Unit::MiB;
    use systemd_run::RunUser;
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/memory");

    #[async_std::test]
    async fn test_memory_ok() {
        let r = RunUser::new(PATH)
            .memory_max(Byte::from_i64_with_unit(384, MiB).unwrap())
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
            .memory_max(Byte::from_i64_with_unit(128, MiB).unwrap())
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

    #[async_std::test]
    #[cfg(feature = "systemd_236")]
    async fn test_slice_memory_limit_exceed() {
        // Create a slice with "unique" name.  I generated it locally with
        // uuidgen.
        const SLICE: &'static str = "7772d908_2631_4b34_aba0_20454e89cf9a.slice";
        let path = std::path::PathBuf::from(env!("XDG_RUNTIME_DIR")).join("systemd/user");

        std::fs::create_dir_all(&path).unwrap();

        let path = path.join(SLICE);
        std::fs::write(&path, b"[Slice]\nMemoryMax=128M\nMemorySwapMax=0\n").unwrap();

        let r = RunUser::new(PATH)
            .slice(SLICE)
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

        // clean up
        RunUser::new("/usr/bin/systemctl")
            .args(&["--user", "stop", SLICE])
            .collect_on_fail()
            .start()
            .await
            .unwrap()
            .wait()
            .await
            .unwrap();
        std::fs::remove_file(&path).unwrap();
    }
}
