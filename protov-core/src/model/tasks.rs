use core::{array::IntoIter, iter::FilterMap};

use super::types::{DisplayTask, HardwareTask};

pub enum Task {
    Hardware(HardwareTask),
    Display(DisplayTask),
}

const APP_TASK_SIZE_LIMIT: usize = 12;

pub struct AppTask {
    pub tasks: [Option<Task>; APP_TASK_SIZE_LIMIT],
    pub count: usize,
}

impl IntoIterator for AppTask {
    type Item = Task;
    type IntoIter =
        FilterMap<IntoIter<Option<Task>, APP_TASK_SIZE_LIMIT>, fn(Option<Task>) -> Option<Task>>;

    fn into_iter(self) -> Self::IntoIter {
        #[allow(clippy::filter_map_identity)]
        self.tasks.into_iter().filter_map(|opt| opt)
    }
}

pub struct AppTaskBuilder {
    inner: AppTask,
}

impl Default for AppTaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppTaskBuilder {
    pub fn new() -> Self {
        Self {
            inner: AppTask {
                tasks: [const { None }; APP_TASK_SIZE_LIMIT],
                count: 0,
            },
        }
    }

    pub fn extend(mut self, other: AppTaskBuilder) -> Self {
        let i = &mut self.inner.count;
        #[allow(clippy::filter_map_identity)]
        let iter = other.inner.tasks.into_iter().filter_map(|opt| opt);

        for task in iter {
            if *i >= APP_TASK_SIZE_LIMIT {
                break;
            }

            self.inner.tasks[*i] = Some(task);
            *i += 1;
        }

        self
    }

    pub fn push_task(mut self, task: Task) -> Self {
        if self.inner.count < self.inner.tasks.len() {
            self.inner.tasks[self.inner.count] = Some(task);
            self.inner.count += 1;
        }

        self
    }

    pub fn hardware(self, task: HardwareTask) -> Self {
        self.push_task(Task::Hardware(task))
    }

    pub fn display(self, task: DisplayTask) -> Self {
        self.push_task(Task::Display(task))
    }

    pub fn build(self) -> Option<AppTask> {
        Some(self.inner)
    }

    pub fn display_task(task: DisplayTask) -> Option<AppTask> {
        let mut tasks = [const { None }; APP_TASK_SIZE_LIMIT];
        tasks[0] = Some(Task::Display(task));

        Some(AppTask { tasks, count: 1 })
    }
}
