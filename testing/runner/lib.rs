use std::{sync::LazyLock, time::Duration};

use console::style;
use crossbeam_channel::{Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use rayon::{ThreadPool, ThreadPoolBuilder};

use rayon::prelude::*;

pub type Result = std::result::Result<(), String>;

type TestImpl = Box<dyn Fn() + 'static + Send>;

static WAITING_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_spinner()
        .template("{prefix}⧖ {msg}")
        .unwrap()
});

static FAILED_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_spinner()
        .template(format!("{{prefix}}{} {{msg}}", style("✗").red()).as_str())
        .unwrap()
});

static TICKING_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    let ticker: Vec<String> = "◐◓◑◒"
        .chars()
        .map(|t| style(t).yellow().to_string())
        .chain([style("✓").green().to_string()])
        .collect();

    let ticker_ref: Vec<&str> = ticker.iter().map(|t| t.as_ref()).collect();

    ProgressStyle::default_spinner()
        .template("{prefix}{spinner} {msg}")
        .unwrap()
        .tick_strings(&ticker_ref)
});

pub enum Test {
    Case {
        name: String,
        code: Box<dyn Fn() -> Result + Send>,
    },
    Suite {
        name: String,
        tests: Vec<Test>,
    },
}

struct TestContext {
    all: usize,
    finished: usize,
    started: bool,
    failed: bool,
    bar_handle: Option<ProgressBar>,
    parent: Option<usize>,
}

impl TestContext {
    fn new(all: usize, bar_handle: ProgressBar, parent: usize) -> Self {
        Self {
            all,
            finished: 0,
            started: false,
            failed: false,
            bar_handle: Some(bar_handle),
            parent: Some(parent),
        }
    }

    fn new_root() -> Self {
        Self {
            all: 1,
            finished: 0,
            started: false,
            failed: false,
            bar_handle: None,
            parent: None,
        }
    }
}

enum Message {
    Started(usize),
    Success(usize),
    Failure(usize, String),
}

struct RunnerState {
    target: MultiProgress,
    cases: Vec<(usize, ProgressBar)>,
    suites: Vec<TestContext>,
    thread_pool: ThreadPool,
    queue: Vec<TestImpl>,
    receiver: Receiver<Message>,
    sender: Sender<Message>,
}

impl RunnerState {
    fn init(test: Test) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Message>();

        let mut state = Self {
            target: MultiProgress::new(),
            cases: Vec::new(),
            suites: Vec::new(),
            thread_pool: ThreadPoolBuilder::new().num_threads(4).build().unwrap(),
            queue: Vec::new(),
            receiver,
            sender,
        };

        state.suites.push(TestContext::new_root());
        state.add_test(test, 0, "".to_string(), "".to_string());

        state
    }

    fn run(self) {
        self.thread_pool.join(
            move || Self::join(self.receiver, self.target, self.suites, self.cases),
            move || {
                let par: rayon::vec::IntoIter<Box<dyn Fn() + Send>> = self.queue.into_par_iter();
                par.for_each(|test| test());
            },
        );
    }

    fn join(
        receiver: Receiver<Message>,
        target: MultiProgress,
        mut suites: Vec<TestContext>,
        cases: Vec<(usize, ProgressBar)>,
    ) {
        while suites[0].finished < suites[0].all {
            let message = receiver.recv().unwrap();
            match message {
                Message::Started(id) => {
                    let (parent, bar) = &cases[id];
                    bar.set_style(TICKING_STYLE.clone());
                    let mut parent_id = Some(*parent);

                    while let Some(parent) = parent_id {
                        let context = &mut suites[parent];
                        if !context.started {
                            if let Some(bar) = context.bar_handle.as_ref() {
                                bar.set_style(TICKING_STYLE.clone());
                            }
                            context.started = true;
                            parent_id = context.parent;
                        } else {
                            parent_id = None;
                        }
                    }
                }
                Message::Success(id) => {
                    let (parent, bar) = &cases[id];
                    bar.finish();
                    let mut parent_id = Some(*parent);

                    while let Some(parent) = parent_id {
                        let context = &mut suites[parent];
                        context.finished += 1;
                        if context.finished == context.all {
                            if let Some(bar) = context.bar_handle.as_ref() {
                                bar.finish();
                            }
                            parent_id = context.parent;
                        } else {
                            parent_id = None;
                        }
                    }
                }
                Message::Failure(id, reason) => {
                    let (parent, bar) = &cases[id];
                    bar.set_style(FAILED_STYLE.clone());
                    let mut parent_id = Some(*parent);

                    while let Some(parent) = parent_id {
                        let context = &mut suites[parent];
                        if !context.failed {
                            context.failed = true;
                            if let Some(bar) = context.bar_handle.as_ref() {
                                bar.set_style(FAILED_STYLE.clone());
                            }
                            parent_id = context.parent;
                        } else {
                            parent_id = None;
                        }
                    }

                    parent_id = Some(*parent);
                    while let Some(parent) = parent_id {
                        let context = &mut suites[parent];
                        context.finished += 1;
                        if context.finished == context.all {
                            if let Some(bar) = context.bar_handle.as_ref() {
                                bar.finish();
                            }
                            parent_id = context.parent;
                        } else {
                            parent_id = None;
                        }
                    }

                    target
                        .println(format!("\n{}", style(reason).red()))
                        .unwrap();
                }
            }
        }
    }

    fn add_test(&mut self, test: Test, parent: usize, prefix: String, child_prefix: String) {
        match test {
            Test::Case { name, code } => {
                let id = self.cases.len();
                let bar = self.target.add(ProgressBar::new_spinner());
                bar.set_style(WAITING_STYLE.clone());
                bar.set_prefix(prefix);
                bar.set_message(name.to_owned());
                bar.enable_steady_tick(Duration::from_millis(75));
                self.cases.push((parent, bar));

                let sender = self.sender.clone();

                self.queue.push(Box::new(move || {
                    sender.send(Message::Started(id)).unwrap();
                    match code() {
                        Ok(_) => {
                            sender.send(Message::Success(id)).unwrap();
                        }
                        Err(e) => {
                            sender.send(Message::Failure(id, e)).unwrap();
                        }
                    }
                }));
            }
            Test::Suite { name, mut tests } => {
                let bar = self.target.add(ProgressBar::new_spinner());
                bar.set_style(WAITING_STYLE.clone());
                bar.set_prefix(prefix.to_owned());
                bar.set_message(name.to_owned());
                bar.enable_steady_tick(Duration::from_millis(75));

                let context_id = self.suites.len();
                self.suites.push(TestContext::new(tests.len(), bar, parent));

                if let Some(last) = tests.pop() {
                    for test in tests {
                        self.add_test(
                            test,
                            context_id,
                            format!("{}├─ ", child_prefix),
                            "│  ".to_string(),
                        );
                    }
                    self.add_test(
                        last,
                        context_id,
                        format!("{}└─ ", child_prefix),
                        "   ".to_string(),
                    );
                }
            }
        }
    }
}

pub fn run_tests(test: Test) {
    RunnerState::init(test).run();
}
