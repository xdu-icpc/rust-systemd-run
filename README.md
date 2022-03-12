# systemd\_run

This is the crate for running processes as
[Systemd](https://systemd.io/) transient services.

**Status:** Highly unstable, at early development cycle.

## Example code

This code starts `/bin/true` as a Systemd transient service, running in
the per-user service manager of your login session, and wait for it to
finish.

```rust,no_run
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let status = systemd_run::Run("/bin/true")
		.start()
		.await?
		.wait()
		.await?;
	dbg!(status);
	Ok(())
}
```
