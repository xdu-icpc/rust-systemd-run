use crate::prelude::*;

pub fn load_file<P: AsRef<Path>>(path: P) -> Result<String> {
    info!(
        "loading file {}",
        path.as_ref().to_str().unwrap_or("[non UTF-8 path]")
    );
    std::fs::read_to_string(path).map_err(Error::IOError)
}

pub fn enumerate_testcase<P: AsRef<Path>>(dir: P) -> Result<Vec<(PathBuf, PathBuf)>> {
    let dir_log = dir.as_ref().display();
    info!("enumerating testcases from {}", dir_log);

    let r = std::fs::read_dir(&dir)
        .map_err(Error::IOError)?
        .filter_map(|x| {
            let x = match x {
                Err(e) => {
                    warn!("error listing {}: {}", dir_log, e);
                    return None;
                }
                Ok(x) => x,
            };
            let p = x.path();
            let name = x.file_name();
            let name = match name.to_str() {
                None => {
                    warn!("skip non-UTF8 file name {} in {}", p.display(), dir_log);
                    return None;
                }
                Some(n) => n,
            };
            name.strip_suffix(".in").map(|x| {
                let outname = x.to_owned() + ".out";
                let dir = dir.as_ref().to_path_buf();
                (p, dir.join(outname))
            })
        })
        .collect::<Vec<_>>();
    Ok(r)
}

pub fn ensure_utf8_path<'a, P: AsRef<Path> + 'a>(p: &'a P) -> Result<&'a str> {
    match p.as_ref().to_str() {
        Some(x) => Ok(x),
        None => Err(Error::BadPathEncoding(p.as_ref().to_path_buf())),
    }
}
