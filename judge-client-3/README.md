# judge-client-3

This is a systemd-based HustOJ `judge_client` drop-in replacement.

## Why number "3"?

The HustOJ instance running on https://acm.xidian.edu.cn/ is our 2nd iteration
for Xidian University Programming Contest Training Base Online Judge.  So we
number this iteration "3".  We'll gradually replace more HustOJ components to
make a 4th iteration.

## How to use

```
DATABASE_URL=mysql://user:password@localhost/jol?socket=/run/mysqld/mysqld.sock cargo build --release --features=hustoj
sudo install -vm755 ../target/release/judge-client-3 /usr/bin
mv /usr/bin/judge_client{,.old}
ln -sv judge-client-3 /usr/bin/judge_client
```

Then custom `judge3.toml` (use `mocktest/etc/judge3.toml` as an example) and
put it into `$OJ_BASE/etc`.

We rely on systemd slices for limiting CPU core numbers.  Custom the files
in `etc/systemd/` and install them into `/etc/systemd`.

And you need to modify HustOJ Web interface code to add "No Output" and
"Judgement Failed" (it has not happened for us) verdicts.

## Advantages

The most significant advantage is we are using a more modern judging
mechanism:

- We don't rely on a very strict system call whitelist.  So we won't suddenly
  blow up after a glibc/libstdc++ upgradation.
- We use an implementation suitable for all languages.  On the contrary, the
  origin `judge_client` needs special case for Java or Python, etc. 
  - So we can support more languages (even MATLAB may be supported with a
    license and a correct configuration, I believe).
  - `judge_client` relies on [deprecated JVM feature][1] for Java.
- We don't use `ptrace`, so we have a better performance for problems with
  many system calls (heavy I/O, for example).
- Our approach is more similar to DOMJudge so we can measure the time usage
  of solutions of problems for onsite contests, and use the result as a good
  approximation.

[1]:https://openjdk.java.net/jeps/411

## Disadvantages

- Only functional with systemd-249 or later (so you need a state-of-art distro
  like Ubuntu 22.04).
- No memory usage measurement or MLE report.
  - Memory limit is implemented but a submission exceeding the limit will be
    judged as RE.
  - Actually there is no reliable way to determine if an RE should be
    considered MLE.  Competitive programming code often has no error handling
	paths so a `NULL` value returned by `malloc`, or a `std::bad_alloc`
	instance will lead to RE instead of MLE.  So the origin HustOJ
	implementation also can't report MLE *reliably*.
- Not as fast as original `judge_client` for very simple C/C++ programs (like
  `hello world` programs).
