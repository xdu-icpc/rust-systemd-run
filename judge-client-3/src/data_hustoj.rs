use crate::data::{Data, DataSource, Verdict};
use crate::prelude::*;

pub struct HustOJDataSource {
    conn: sqlx::MySqlConnection,
    oj_home: PathBuf,
}

#[derive(Debug)]
struct QueryLine {
    source: String,
    problem_id: i32,
    time_limit: i32,
    memory_limit: i32,
    spj: u8,
    result: i16,
    language: u32,
}

impl HustOJDataSource {
    pub fn new<P: AsRef<Path>>(conn: sqlx::MySqlConnection, oj_home: P) -> Self {
        Self {
            conn,
            oj_home: oj_home.as_ref().into(),
        }
    }
}

#[async_trait::async_trait]
impl DataSource for HustOJDataSource {
    async fn fetch<T: AsRef<str> + Send>(&mut self, id: T) -> Result<Data> {
        let id: i32 = id
            .as_ref()
            .parse()
            .map_err(|_| Error::BadSolutionID(id.as_ref().to_owned()))?;
        let line: QueryLine = sqlx::query_as_unchecked!(
            QueryLine,
            "SELECT solution.problem_id, \
                    solution.result, \
                    solution.language, \
                    source_code.source, \
                    problem.time_limit, \
                    problem.memory_limit, \
                    problem.spj \
             FROM solution, source_code, problem \
             WHERE source_code.solution_id = ? \
               AND source_code.solution_id = solution.solution_id \
               AND solution.problem_id = problem.problem_id",
            id
        )
        .fetch_one(&mut self.conn)
        .await
        .map_err(Error::SQLError)?;

        let p = line.problem_id;

        let time_limit = u64::try_from(line.time_limit).map_err(|_| Error::BadProblem(p))?;
        if time_limit == 0 {
            return Err(Error::BadProblem(p));
        }
        let time_limit = Duration::from_secs(time_limit);

        let memory_limit = u64::try_from(line.memory_limit).map_err(|_| Error::BadProblem(p))?;
        if memory_limit == 0 {
            return Err(Error::BadProblem(p));
        }

        let memory_limit = Byte::from_bytes(memory_limit as u128 * byte_unit::MEBIBYTE);

        let language = match line.language {
            0 => "c",
            1 => "c++",
            2 => "pascal",
            3 => "java",
            _ => {
                let l = format!("language with HUST ID {}", line.language);
                return Err(Error::UnconfiguredLanguage(l));
            }
        }
        .to_string();

        let old_result = match line.result {
            4 => Some(Verdict::Correct),
            5 | 6 => Some(Verdict::WrongAnswer),
            7 => Some(Verdict::TimeLimit),
            8 | 10 => Some(Verdict::RunError),
            9 => Some(Verdict::OutputLimit),
            11 => Some(Verdict::CompilerError),
            _ => None,
        };

        let data_dir = self.oj_home.join("data").join(p.to_string());
        let testcases = util::enumerate_testcase(&data_dir)?;

        // Stupid enough, HUSTOJ uses CHAR(1) for SPJ, instead of a rational
        // BOOLEAN or TINYINT(1).
        let spj = match line.spj {
            b'1' => Some(data_dir.join("spj")),
            _ => None,
        };

        Ok(Data {
            time_limit,
            memory_limit,
            language,
            old_result,
            source: line.source.into_bytes(),
            spj,
            testcases,
        })
    }
    async fn feedback<T: AsRef<str> + Send>(
        &mut self,
        _id: T,
        _v: Verdict,
        _d: Duration,
    ) -> Result<()> {
        error!("HustOJDataSource::feedback not implemented yet");
        Ok(())
    }
    async fn feedback_ce<T: AsRef<str> + Send>(&mut self, _id: T, _msg: Vec<u8>) -> Result<()> {
        error!("HustOJDataSource::feedback_ce not implemented yet");
        Ok(())
    }
    async fn feedback_log<T: AsRef<str> + Send>(&mut self, _id: T, _msg: Vec<u8>) -> Result<()> {
        error!("HustOJDataSource::feedback_log not implemented yet");
        Ok(())
    }
}
