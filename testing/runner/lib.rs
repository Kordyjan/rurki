use std::panic::{catch_unwind, resume_unwind, UnwindSafe};
use std::{process, result::Result as StdResult, sync::LazyLock, time::Duration};

use anyhow::Error;
use console::style;
use crossbeam_channel::{after, select, Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use model::Test;
use rayon::{ThreadPool, ThreadPoolBuilder};

use rayon::prelude::*;

pub mod model;

pub type Result = anyhow::Result<()>;

type TestImpl = Box<dyn FnOnce() + 'static + Send>;

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

    let ticker_ref: Vec<&str> = ticker.iter().map(AsRef::as_ref).collect();

    ProgressStyle::default_spinner()
        .template("{prefix}{spinner} {msg}")
        .unwrap()
        .tick_strings(&ticker_ref)
});

struct SuiteContext {
    all: usize,
    finished: usize,
    started: bool,
    failed: bool,
    bar_handle: Option<ProgressBar>,
    parent: Option<usize>,
}

impl SuiteContext {
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

struct TestContext {
    name: String,
    parent: usize,
    bar_handle: ProgressBar,
}

enum Message {
    Started(usize),
    Success(usize),
    Failure(usize, Error),
}

struct RunnerState {
    target: MultiProgress,
    cases: Vec<TestContext>,
    suites: Vec<SuiteContext>,
    thread_pool: ThreadPool,
    queue: Vec<TestImpl>,
    receiver: Receiver<Message>,
    sender: Sender<Message>,
}

impl RunnerState {
    fn init<T: 'static>(
        test: Test<T>,
        tested_data: impl Fn() -> T + Send + Clone + UnwindSafe + 'static,
    ) -> Self {
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

        state.suites.push(SuiteContext::new_root());
        state.add_test(test, 0, String::new(), String::new(), tested_data);

        state
    }

    fn run(self, timeout: Duration) {
        self.thread_pool.join(
            move || Self::join(self.receiver, self.target, self.suites, self.cases, timeout),
            move || {
                let par: rayon::vec::IntoIter<Box<dyn FnOnce() + Send>> =
                    self.queue.into_par_iter();
                par.for_each(|test| test());
            },
        );
    }

    fn join(
        receiver: Receiver<Message>,
        target: MultiProgress,
        mut suites: Vec<SuiteContext>,
        cases: Vec<TestContext>,
        timeout: Duration,
    ) {
        let timer = after(timeout);
        while suites[0].finished < suites[0].all {
            let message: StdResult<Message, ()> = select! {
                recv(receiver) -> msg => Ok(msg.unwrap()),
                recv(timer) -> _ => Err(()),
            };

            if let Ok(msg) = message {
                Self::handle_join_message(msg, &mut suites, &cases, &target);
            } else {
                for TestContext { bar_handle, .. } in &cases {
                    if !bar_handle.is_finished() {
                        bar_handle.set_style(FAILED_STYLE.clone());
                        bar_handle.finish();
                    }
                }

                for SuiteContext {
                    bar_handle,
                    finished,
                    all,
                    ..
                } in &mut suites
                {
                    *finished = *all;
                    if let Some(bar_handle) = bar_handle {
                        if !bar_handle.is_finished() {
                            bar_handle.set_style(FAILED_STYLE.clone());
                            bar_handle.finish();
                        }
                    }
                }

                eprintln!("Timeout!");
                process::exit(1);
            }
        }
    }

    fn handle_join_message(
        msg: Message,
        suites: &mut [SuiteContext],
        cases: &[TestContext],
        target: &MultiProgress,
    ) {
        match msg {
            Message::Started(id) => {
                let TestContext {
                    parent,
                    bar_handle: bar,
                    ..
                } = &cases[id];
                bar.set_style(TICKING_STYLE.clone());
                let mut parent_id = Some(*parent);

                while let Some(parent) = parent_id {
                    let context = &mut suites[parent];
                    if context.started {
                        parent_id = None;
                    } else {
                        if let Some(bar) = context.bar_handle.as_ref() {
                            bar.set_style(TICKING_STYLE.clone());
                        }
                        context.started = true;
                        parent_id = context.parent;
                    }
                }
            }
            Message::Success(id) => {
                let TestContext {
                    parent,
                    bar_handle: bar,
                    ..
                } = &cases[id];
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
                let TestContext {
                    name,
                    parent,
                    bar_handle: bar,
                } = &cases[id];
                bar.set_style(FAILED_STYLE.clone());
                bar.finish();

                let mut parent_id = Some(*parent);
                while let Some(parent) = parent_id {
                    let context = &mut suites[parent];
                    if context.failed {
                        parent_id = None;
                    } else {
                        context.failed = true;
                        if let Some(bar) = context.bar_handle.as_ref() {
                            bar.set_style(FAILED_STYLE.clone());
                        }
                        parent_id = context.parent;
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
                    .println(format!("{}: {}\n", style(name).red(), reason))
                    .unwrap();
            }
        }
    }

    fn add_test<T: 'static>(
        &mut self,
        test: Test<T>,
        parent: usize,
        prefix: String,
        child_prefix: String,
        tested_data: impl Fn() -> T + Send + Clone + UnwindSafe + 'static,
    ) {
        match test {
            Test::Case { name, code } => {
                let id = self.cases.len();
                let bar = self.target.add(ProgressBar::new_spinner());
                bar.set_style(WAITING_STYLE.clone());
                bar.set_prefix(prefix);
                bar.set_message(name.clone());
                bar.enable_steady_tick(Duration::from_millis(75));
                self.cases.push(TestContext {
                    name,
                    parent,
                    bar_handle: bar,
                });

                let sender = self.sender.clone();

                let data = tested_data.clone();
                self.queue.push(Box::new(move || {
                    sender.send(Message::Started(id)).unwrap();
                    let sender2 = sender.clone();
                    let caught = catch_unwind(move || match code(data()) {
                        Ok(()) => {
                            sender.send(Message::Success(id)).unwrap();
                        }
                        Err(e) => {
                            sender.send(Message::Failure(id, e)).unwrap();
                        }
                    });
                    if let Err(e) = caught {
                        if let Some(s) = e.downcast_ref::<String>() {
                            sender2
                                .send(Message::Failure(id, Error::msg(s.clone())))
                                .unwrap();
                        } else {
                            sender2
                                .send(Message::Failure(
                                    id,
                                    Error::msg("Thread panicked with unknown error. Check stderr."),
                                ))
                                .unwrap();
                            resume_unwind(e);
                        }
                    };
                }));
            }
            Test::Suite { name, mut tests } => {
                let bar = self.target.add(ProgressBar::new_spinner());
                bar.set_style(WAITING_STYLE.clone());
                bar.set_prefix(prefix.clone());
                bar.set_message(name.clone());
                bar.enable_steady_tick(Duration::from_millis(75));

                let context_id = self.suites.len();
                self.suites
                    .push(SuiteContext::new(tests.len(), bar, parent));

                if let Some(last) = tests.pop() {
                    for test in tests {
                        self.add_test(
                            test,
                            context_id,
                            format!("{child_prefix}├─ "),
                            "│  ".to_string(),
                            tested_data.clone(),
                        );
                    }
                    self.add_test(
                        last,
                        context_id,
                        format!("{child_prefix}└─ "),
                        "   ".to_string(),
                        tested_data.clone(),
                    );
                }
            }
        }
    }
}

pub fn run_tests<T: 'static>(
    test: Test<T>,
    tested_data: impl Fn() -> T + Send + Clone + UnwindSafe + 'static,
    timeout: Duration,
) {
    RunnerState::init(test, tested_data).run(timeout);
}
