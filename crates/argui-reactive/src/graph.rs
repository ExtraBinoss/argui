use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::ReactiveError;

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_node_id() -> u64 {
    NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed)
}

pub(crate) struct Dependent {
    pub node: Weak<dyn Node>,
    pub generation: u64,
}

pub(crate) trait Node {
    fn id(&self) -> u64;
    fn add_dependent(&self, dependent: Dependent);
    fn mark_dirty(&self, generation: u64) -> bool;
    fn flush(&self);
}

struct Collector {
    dependent: Weak<dyn Node>,
    generation: u64,
    seen: HashSet<u64>,
}

#[derive(Default)]
struct Scheduler {
    transaction_depth: usize,
    flushing: bool,
    queued: HashSet<u64>,
    queue: VecDeque<Rc<dyn Node>>,
}

thread_local! {
    static COLLECTORS: RefCell<Vec<Collector>> = const { RefCell::new(Vec::new()) };
    static EVALUATING: RefCell<Vec<(u64, String)>> = const { RefCell::new(Vec::new()) };
    static SCHEDULER: RefCell<Scheduler> = RefCell::new(Scheduler::default());
}

pub(crate) fn track(source: Rc<dyn Node>) {
    COLLECTORS.with(|collectors| {
        let mut collectors = collectors.borrow_mut();
        let Some(collector) = collectors.last_mut() else {
            return;
        };
        if collector.seen.insert(source.id()) {
            source.add_dependent(Dependent {
                node: collector.dependent.clone(),
                generation: collector.generation,
            });
        }
    });
}

pub(crate) fn collect<R>(
    dependent: Weak<dyn Node>,
    generation: u64,
    operation: impl FnOnce() -> R,
) -> R {
    COLLECTORS.with(|collectors| {
        collectors.borrow_mut().push(Collector {
            dependent,
            generation,
            seen: HashSet::new(),
        });
    });
    struct PopCollector;
    impl Drop for PopCollector {
        fn drop(&mut self) {
            COLLECTORS.with(|collectors| {
                collectors.borrow_mut().pop();
            });
        }
    }
    let guard = PopCollector;
    let result = operation();
    drop(guard);
    result
}

pub(crate) fn enter_evaluation<R>(
    id: u64,
    label: &str,
    operation: impl FnOnce() -> R,
) -> Result<R, ReactiveError> {
    let cycle = EVALUATING.with(|evaluating| {
        let evaluating = evaluating.borrow();
        evaluating
            .iter()
            .position(|(candidate, _)| *candidate == id)
            .map(|start| {
                evaluating[start..]
                    .iter()
                    .map(|(_, label)| label.clone())
                    .chain(std::iter::once(label.to_owned()))
                    .collect::<Vec<_>>()
            })
    });
    if let Some(path) = cycle {
        return Err(ReactiveError::Cycle { path });
    }
    EVALUATING.with(|evaluating| evaluating.borrow_mut().push((id, label.to_owned())));
    struct PopEvaluation;
    impl Drop for PopEvaluation {
        fn drop(&mut self) {
            EVALUATING.with(|evaluating| {
                evaluating.borrow_mut().pop();
            });
        }
    }
    let guard = PopEvaluation;
    let result = operation();
    drop(guard);
    Ok(result)
}

pub(crate) fn invalidate(dependents: &RefCell<Vec<Dependent>>) {
    let mut pending = Vec::new();
    dependents.borrow_mut().retain(|dependent| {
        let Some(node) = dependent.node.upgrade() else {
            return false;
        };
        if !node.mark_dirty(dependent.generation) {
            return false;
        }
        pending.push(node);
        true
    });
    for node in pending {
        schedule(node);
    }
}

pub(crate) fn schedule(node: Rc<dyn Node>) {
    let flush = SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        if scheduler.queued.insert(node.id()) {
            scheduler.queue.push_back(node);
        }
        scheduler.transaction_depth == 0 && !scheduler.flushing
    });
    if flush {
        flush_queue();
    }
}

fn flush_queue() {
    SCHEDULER.with(|scheduler| scheduler.borrow_mut().flushing = true);
    loop {
        let node = SCHEDULER.with(|scheduler| {
            let mut scheduler = scheduler.borrow_mut();
            let node = scheduler.queue.pop_front();
            if let Some(node) = &node {
                scheduler.queued.remove(&node.id());
            }
            node
        });
        let Some(node) = node else {
            break;
        };
        node.flush();
    }
    SCHEDULER.with(|scheduler| scheduler.borrow_mut().flushing = false);
}

pub(crate) fn begin_transaction() {
    SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        scheduler.transaction_depth = scheduler.transaction_depth.saturating_add(1);
    });
}

pub(crate) fn end_transaction() {
    let flush = SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        scheduler.transaction_depth = scheduler.transaction_depth.saturating_sub(1);
        scheduler.transaction_depth == 0 && !scheduler.flushing && !scheduler.queue.is_empty()
    });
    if flush {
        flush_queue();
    }
}
