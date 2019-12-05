use crate::{Checkpoint, Command, History, Record, Result, Signal, Timeline};
use alloc::vec::Vec;

/// A command queue wrapper.
///
/// Wraps a record or history and gives it batch queue functionality.
/// This allows the record or history to queue up commands and either cancel or apply them later.
///
/// # Examples
/// ```
/// # use redo::{Command, Record};
/// # struct Add(char);
/// # impl Command for Add {
/// #     type Target = String;
/// #     type Error = &'static str;
/// #     fn apply(&mut self, s: &mut String) -> redo::Result<Add> {
/// #         s.push(self.0);
/// #         Ok(())
/// #     }
/// #     fn undo(&mut self, s: &mut String) -> redo::Result<Add> {
/// #         self.0 = s.pop().ok_or("`s` is empty")?;
/// #         Ok(())
/// #     }
/// # }
/// # fn main() -> redo::Result<Add> {
/// let mut record = Record::default();
/// let mut queue = record.queue();
/// queue.apply(Add('a'));
/// queue.apply(Add('b'));
/// queue.apply(Add('c'));
/// assert_eq!(queue.target(), "");
/// queue.commit()?;
/// assert_eq!(record.target(), "abc");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Queue<'a, T: Timeline> {
    inner: &'a mut T,
    actions: Vec<Action<T::Command>>,
}

impl<'a, T: Timeline> Queue<'a, T> {
    /// Returns a queue.
    #[inline]
    pub fn new(inner: &'a mut T) -> Queue<'a, T> {
        Queue {
            inner,
            actions: Vec::new(),
        }
    }

    /// Reserves capacity for at least `additional` more commands in the queue.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.actions.reserve(additional);
    }

    /// Returns the capacity of the queue.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.actions.capacity()
    }

    /// Shrinks the capacity of the queue as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.actions.shrink_to_fit();
    }

    /// Returns the number of commands in the queue.
    #[inline]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns `true` if the queue is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Queues an `apply` action.
    #[inline]
    pub fn apply(&mut self, command: T::Command) {
        self.actions.push(Action::Apply(command));
    }

    /// Queues an `undo` action.
    #[inline]
    pub fn undo(&mut self) {
        self.actions.push(Action::Undo);
    }

    /// Queues a `redo` action.
    #[inline]
    pub fn redo(&mut self) {
        self.actions.push(Action::Redo);
    }

    /// Queues an `apply` action for each command in the iterator.
    #[inline]
    pub fn extend(&mut self, commands: impl IntoIterator<Item = T::Command>) {
        for command in commands {
            self.apply(command);
        }
    }

    /// Cancels the queued actions.
    #[inline]
    pub fn cancel(self) {}
}

impl<C: Command, F: FnMut(Signal)> Queue<'_, Record<C, F>> {
    /// Queues a `go_to` action.
    #[inline]
    pub fn go_to(&mut self, current: usize) {
        self.actions.push(Action::GoTo(0, current));
    }

    /// Applies the actions that is queued.
    ///
    /// # Errors
    /// If an error occurs, it stops applying the actions and returns the error.
    #[inline]
    pub fn commit(self) -> Result<C> {
        for action in self.actions {
            match action {
                Action::Apply(command) => self.inner.apply(command)?,
                Action::Undo => {
                    if let Some(Err(error)) = self.inner.undo() {
                        return Err(error);
                    }
                }
                Action::Redo => {
                    if let Some(Err(error)) = self.inner.redo() {
                        return Err(error);
                    }
                }
                Action::GoTo(_, current) => {
                    if let Some(Err(error)) = self.inner.go_to(current) {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns a queue.
    #[inline]
    pub fn queue(&mut self) -> Queue<Record<C, F>> {
        self.inner.queue()
    }

    /// Returns a checkpoint.
    #[inline]
    pub fn checkpoint(&mut self) -> Checkpoint<Record<C, F>> {
        self.inner.checkpoint()
    }

    /// Returns a reference to the `target`.
    #[inline]
    pub fn target(&self) -> &C::Target {
        self.inner.target()
    }

    /// Returns a mutable reference to the `target`.
    ///
    /// This method should **only** be used when doing changes that should not be able to be undone.
    #[inline]
    pub fn target_mut(&mut self) -> &mut C::Target {
        self.inner.target_mut()
    }
}

impl<C: Command, F: FnMut(Signal)> Queue<'_, History<C, F>> {
    /// Queues a `go_to` action.
    #[inline]
    pub fn go_to(&mut self, branch: usize, current: usize) {
        self.actions.push(Action::GoTo(branch, current));
    }

    /// Applies the actions that is queued.
    ///
    /// # Errors
    /// If an error occurs, it stops applying the actions and returns the error.
    #[inline]
    pub fn commit(self) -> Result<C> {
        for action in self.actions {
            match action {
                Action::Apply(command) => self.inner.apply(command)?,
                Action::Undo => {
                    if let Some(Err(error)) = self.inner.undo() {
                        return Err(error);
                    }
                }
                Action::Redo => {
                    if let Some(Err(error)) = self.inner.redo() {
                        return Err(error);
                    }
                }
                Action::GoTo(branch, current) => {
                    if let Some(Err(error)) = self.inner.go_to(branch, current) {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns a queue.
    #[inline]
    pub fn queue(&mut self) -> Queue<History<C, F>> {
        self.inner.queue()
    }

    /// Returns a checkpoint.
    #[inline]
    pub fn checkpoint(&mut self) -> Checkpoint<History<C, F>> {
        self.inner.checkpoint()
    }

    /// Returns a reference to the `target`.
    #[inline]
    pub fn target(&self) -> &C::Target {
        self.inner.target()
    }

    /// Returns a mutable reference to the `target`.
    ///
    /// This method should **only** be used when doing changes that should not be able to be undone.
    #[inline]
    pub fn target_mut(&mut self) -> &mut C::Target {
        self.inner.target_mut()
    }
}

impl<T: Timeline> Timeline for Queue<'_, T> {
    type Command = T::Command;

    #[inline]
    fn apply(&mut self, command: T::Command) -> Result<T::Command> {
        self.apply(command);
        Ok(())
    }

    #[inline]
    fn undo(&mut self) -> Option<Result<T::Command>> {
        self.undo();
        Some(Ok(()))
    }

    #[inline]
    fn redo(&mut self) -> Option<Result<T::Command>> {
        self.redo();
        Some(Ok(()))
    }
}

impl<'a, T: Timeline> From<&'a mut T> for Queue<'a, T> {
    #[inline]
    fn from(inner: &'a mut T) -> Self {
        Queue::new(inner)
    }
}

/// An action that can be applied to a Record or History.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum Action<C> {
    Apply(C),
    Undo,
    Redo,
    GoTo(usize, usize),
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::string::String;

    struct Add(char);

    impl Command for Add {
        type Target = String;
        type Error = &'static str;

        fn apply(&mut self, s: &mut String) -> Result<Add> {
            s.push(self.0);
            Ok(())
        }

        fn undo(&mut self, s: &mut String) -> Result<Add> {
            self.0 = s.pop().ok_or("`s` is empty")?;
            Ok(())
        }
    }

    #[test]
    fn commit() {
        let mut record = Record::default();
        let mut q1 = record.queue();
        q1.redo();
        q1.redo();
        q1.redo();
        let mut q2 = q1.queue();
        q2.undo();
        q2.undo();
        q2.undo();
        let mut q3 = q2.queue();
        q3.apply(Add('a'));
        q3.apply(Add('b'));
        q3.apply(Add('c'));
        assert_eq!(q3.target(), "");
        q3.commit().unwrap();
        assert_eq!(q2.target(), "abc");
        q2.commit().unwrap();
        assert_eq!(q1.target(), "");
        q1.commit().unwrap();
        assert_eq!(record.target(), "abc");
    }
}
