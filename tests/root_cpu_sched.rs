use std::fs::read_to_string;
use systemd_run::{CpuScheduling, OutputSpec, RunSystem};

#[ignore]
#[async_std::test]
async fn test_root_cpu_sched() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/rust-systemd-run-test");
    let sched = CpuScheduling::round_robin(42.try_into().unwrap());
    let r = RunSystem::new("/usr/bin/chrt")
        .arg("-p")
        .arg("0")
        .stdout(OutputSpec::file(PATH))
        .cpu_schedule(sched)
        .start()
        .await
        .expect("should be able to start /usr/bin/chrt -p 0")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "chrt -p 0 should run successfully");

    let content = read_to_string(PATH).expect("shoule be able to read chrt -p 0 output");

    assert!(content.contains("SCHED_RR\n"), "wrong schedule policy");
    assert!(content.contains("42\n"), "wrong real-time priority");
}
