use crate::data::{Data, DataSource, Verdict};
use crate::prelude::*;

#[serde_with::serde_as]
#[derive(Deserialize)]
struct DataFile {
    pub language: String,
    #[serde_as(as = "serde_with::DurationSeconds<f64>")]
    pub time_limit: Duration,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub memory_limit: Byte,
    pub spj: Option<PathBuf>,
    pub testcase_dir: PathBuf,
    pub src: PathBuf,
    pub expect: Verdict,
}

impl DataFile {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = util::load_file(path)?;
        toml::from_str(&content).map_err(Error::TOMLParseError)
    }

    fn into_data(self) -> Result<Data> {
        let source = std::fs::read(self.src).map_err(Error::IOError)?;
        let testcases = util::enumerate_testcase(&self.testcase_dir)?;
        Ok(Data {
            source,
            language: self.language,
            time_limit: self.time_limit,
            memory_limit: self.memory_limit,
            spj: self.spj,
            old_result: Some(self.expect),
            testcases,
        })
    }
}

pub struct MockDataSource {}

impl MockDataSource {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl DataSource for MockDataSource {
    async fn fetch<T: AsRef<str> + Send>(&mut self, id: T) -> Result<Data> {
        let f = id.as_ref().to_owned() + ".toml";
        DataFile::load(f)?.into_data()
    }
    async fn feedback<T: AsRef<str> + Send>(
        &mut self,
        _id: T,
        _v: Verdict,
        _d: Duration,
    ) -> Result<()> {
        Ok(())
    }
    async fn feedback_ce<T: AsRef<str> + Send>(&mut self, id: T, msg: Vec<u8>) -> Result<()> {
        let name = "output/".to_owned() + id.as_ref() + ".compile.txt";
        std::fs::write(name, &msg).map_err(Error::IOError)
    }
    async fn feedback_log<T: AsRef<str> + Send>(&mut self, id: T, msg: Vec<u8>) -> Result<()> {
        let name = "output/".to_owned() + id.as_ref() + ".judgelog.txt";
        std::fs::write(name, &msg).map_err(Error::IOError)
    }
}
