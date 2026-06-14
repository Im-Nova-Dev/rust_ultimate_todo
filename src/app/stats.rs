use crate::model::{Status, Task};

pub(crate) fn count_all(tasks: &[Task]) -> usize {
    let mut n = tasks.len();
    for t in tasks {
        n += count_all(&t.children);
    }
    n
}

pub(crate) fn count_done(tasks: &[Task]) -> usize {
    let mut n = tasks.iter().filter(|t| t.status == Status::Done).count();
    for t in tasks {
        n += count_done(&t.children);
    }
    n
}
