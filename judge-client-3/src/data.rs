use crate::prelude::*;

/// Possible judge results, mostly aligned with
/// [DOMJudge](https://www.domjudge.org/docs/team-manual.pdf).
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Verdict {
    Correct,
    CompilerError,
    TimeLimit,
    RunError,
    NoOutput,
    OutputLimit,
    WrongAnswer,
    JudgementFailed,
}

#[derive(Debug)]
pub struct Data {
    /// Source code content, not path
    pub source: Vec<u8>,
    /// Language
    pub language: String,
    /// Time limit
    pub time_limit: Duration,
    /// Memory limit
    pub memory_limit: Byte,
    /// SPJ executable path
    pub spj: Option<PathBuf>,
    /// [("/path/to/in", "/path/to/ans")]
    pub testcases: Vec<(PathBuf, PathBuf)>,
    /// Old result if exists
    pub old_result: Option<Verdict>,
}

pub trait DataSource {
    fn fetch<T: AsRef<str>>(&mut self, id: T) -> Result<Data>;
    fn feedback<T: AsRef<str>>(&mut self, id: T, v: Verdict) -> Result<()>;
    fn feedback_ce<T: AsRef<str>>(&mut self, id: T, msg: Vec<u8>) -> Result<()>;
    fn feedback_log<T: AsRef<str>>(&mut self, id: T, msg: Vec<u8>) -> Result<()>;
}
