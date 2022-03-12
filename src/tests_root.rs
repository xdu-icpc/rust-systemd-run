use crate::{Identity, Run};

#[async_std::test]
#[ignore]
async fn test_dynamic_user() {
    let r = Run::new("/bin/true")
        .identity(Identity::Dynamic)
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        !r.is_failed(),
        "/bin/true should run successfully with Dynamic Identity"
    );
}

#[async_std::test]
#[ignore]
async fn test_nobody() {
    let r = Run::new("/bin/true")
        .identity(Identity::user("nobody"))
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        !r.is_failed(),
        "/bin/true should run successfully with nobody Identity"
    );
}

#[async_std::test]
#[ignore]
async fn test_dynamic_user_access() {
    let f = "/run/rust_systemd_run_test_file";
    let r = Run::new("/bin/touch")
        .arg(f)
        .identity(Identity::Dynamic)
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "Dynamic identity should not be able to touch {}",
        f
    );
}

#[async_std::test]
#[ignore]
async fn test_nobody_access() {
    let f = "/run/rust_systemd_run_test_file";
    let r = Run::new("/bin/touch")
        .arg(f)
        .identity(Identity::user("nobody"))
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "nobody Identity should not be able to touch {}",
        f
    );
}
