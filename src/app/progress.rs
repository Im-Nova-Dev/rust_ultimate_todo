use super::*;

impl App {
    pub(crate) fn subtask_progress(&self, task: &Task) -> (usize, usize) {
        fn count(t: &Task, depth: usize) -> (usize, usize) {
            if depth > 128 {
                return (0, 1);
            }
            let mut done = if t.status == Status::Done { 1 } else { 0 };
            let mut total = 1;
            for c in &t.children {
                let (d, tot) = count(c, depth + 1);
                done += d;
                total += tot;
            }
            (done, total)
        }
        count(task, 0)
    }
}
