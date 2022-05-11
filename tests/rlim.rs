use byte_unit::Byte;
use systemd_run::RunUser;

#[async_std::test]
#[cfg(feature = "systemd_236")]
async fn test_limit_fsize() {
    const F: &'static str = concat!(env!("OUT_DIR"), "/test-aux/test-fsz");
    // Attempt to copy 4M, but use limit_fsize = 1M to stop it.
    let lim = Byte::from_str("1 MiB").unwrap();
    let r = RunUser::new("/bin/dd")
        .arg("if=/dev/zero")
        .arg("of=".to_owned() + F)
        .arg("bs=4096")
        .arg("count=1024")
        .limit_fsize(lim)
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start test dd")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(r.is_failed(), "dd shouldn't be able to finish the write");

    let f = std::fs::File::open(F).expect("output file should exist");
    use std::os::unix::fs::MetadataExt;
    let meta = f.metadata().expect("should be able to get the metadata");
    assert!(Byte::from_bytes(meta.size() as u128) <= lim);
}
