use byte_unit::Byte;
use systemd_run::Run;

#[async_std::test]
async fn test_memory_ok() {
    const PATH: &'static str = env!("CARGO_BIN_EXE_memory");
    let r = Run::new(PATH)
        .memory_max(Byte::from_str("384 MB").unwrap())
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
async fn test_memory_limit_exceed() {
    const PATH: &'static str = env!("CARGO_BIN_EXE_memory");
    let r = Run::new(PATH)
        .memory_max(Byte::from_str("128 MB").unwrap())
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
