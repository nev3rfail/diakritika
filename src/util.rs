use std::iter::once;
use std::time::{Duration, Instant};
use log::{Level, logger};

pub fn measure_and_return<T, F>(f: F) -> (T, Duration)
    where
        F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f(); // Execute the closure and store the result
    let duration = Instant::now() - start; // Measure the time taken
    (result, duration) // Return the result and the duration
}

pub fn measure<T, F>(f: F) -> T
    where
        F: FnOnce() -> T,
{
    let (r, duration) = measure_and_return(f);
    println!("Execution time: {:?}", duration);
    r
}

pub enum ProfilerLogger {
    Static(fn(&str)),
    Boxed(Box<dyn Fn(&str)>),
    Noop
}

impl ProfilerLogger {
    pub fn log(&self, message: &str) {
        match self {
            ProfilerLogger::Static(f) => f(message),
            ProfilerLogger::Boxed(f) => f(message),
            _ => {}
        }
    }
}

type BoxedProfilerLogger = Box<dyn Fn(&str)>;
type ConstProfilerLogger = fn(&str);
pub struct ProfilerFactory<'a> {
    pub(crate) prefix: &'a str,
    pub(crate) logger: ProfilerLogger,
}


impl <'a>ProfilerFactory<'a> {

    pub(crate) fn new(prefix: &'a str, logger: ProfilerLogger) -> Self {
        ProfilerFactory { prefix, logger: logger }
    }

    pub const fn static_new(prefix: &'a str, logger: ConstProfilerLogger) -> Self {
        //let s =Box::new("");

        ProfilerFactory { prefix, logger: ProfilerLogger::Static(logger) }
    }

    // Invoke operator to create a Profiler
    pub fn dispense(&self, message: &str) -> Profiler {
        Profiler::new(message.to_string(), self.prefix, &self.logger)
    }
    pub(crate) fn oneshot<F, T>(&self, msg: &str, func: F) -> T
        where
            F: FnOnce(Profiler) -> T,
    {
        let s = self.dispense(msg);
        let r = func(s);
        r
    }
}


pub struct Profiler<'a> {
    message: String,
    prefix: &'a str,
    logger: &'a ProfilerLogger,
    start: Instant,
}

impl <'a> Profiler<'a> {
    fn new(message: String, prefix: &'a str, logger: &'a ProfilerLogger) -> Self {
        let profiler = Profiler {
            message,
            prefix,
            logger,
            start: Instant::now(),
        };
        profiler.log("started");
        profiler
    }

    pub fn log(&self, text: &str) {
        self.logger.log(&format!("[⏲] [{}] {} {}", self.prefix, self.message, text));
    }

    pub fn elapsed(&self) {
        self.log(&format!("running for {:?}", self.start.elapsed()));
    }

    pub(crate) fn r#do<F, T>(&self, func: F) -> T
        where
            F: FnOnce(&Self) -> T,
    {
        func(self)
    }

}

impl <'a> Drop for Profiler<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.log(&format!("finished in {:?}", duration));
    }
}


pub const fn default_logger(level: Level) -> fn(&str) {
    match level {
        Level::Error =>|str|{ log::error!("{}", str) },
        Level::Warn => |str|{ log::warn!("{}", str) },
        Level::Info => |str|{ log::info!("{}", str) },
        Level::Debug => |str|{ log::debug!("{}", str) },
        Level::Trace => |str|{ log::trace!("{}", str) },
    }
}