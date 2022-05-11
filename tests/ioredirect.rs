use systemd_run::{InputSpec, OutputSpec, RunUser};

#[async_std::test]
#[cfg(feature = "systemd_236")]
async fn test_all_null() {
    const EXE: &'static str = concat!(env!("OUT_DIR"), "/test-aux/rw");
    let r = RunUser::new(EXE)
        .arg("r")
        .stdin(InputSpec::null())
        .stdout(OutputSpec::null())
        .stderr(OutputSpec::null())
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start test program rw")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "'rw r' should not run successfully w/o input"
    );
}

#[async_std::test]
async fn test_stdin_file() {
    const EXE: &'static str = concat!(env!("OUT_DIR"), "/test-aux/rw");
    const DATA: &'static str = concat!(env!("OUT_DIR"), "/test-aux/stdin.txt");
    let r = RunUser::new(EXE)
        .arg("r")
        .stdin(InputSpec::file(DATA))
        .start()
        .await
        .expect("should be able to start test program rw")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "'rw r' should have run successfully");
}
