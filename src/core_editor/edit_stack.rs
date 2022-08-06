#[derive(Debug, PartialEq, Eq)]

/// Represent a list of pending undos and redos.
/// Note: Does not track the active state. The caller must track the active
/// state, which helps to minimize clone operations
pub struct EditStack<T> {
    undo_list: Vec<T>,
    redo_list: Vec<T>,
}

impl<T> EditStack<T> {
    pub fn new() -> Self
    where
        T: Default,
    {
        EditStack {
            undo_list: Vec::new(),
            redo_list: Vec::new(),
        }
    }
}

impl<T> EditStack<T>
where
    T: Default + Clone + Send,
{
    /// Go back one point in the undo stack. If present on first edit do nothing
    /// Updates the current_state parameter in-place if an undo is possible
    pub(super) fn undo(&mut self, current_state: &mut T) {
        if let Some(prev_state) = self.undo_list.pop() {
            self.redo_list
                .push(std::mem::replace(current_state, prev_state));
        }
    }

    /// Go forward one point in the undo stack. If present on the last edit do nothing
    /// Updates the current_state parameter in-place if a redo is possible
    pub(super) fn redo(&mut self, current_state: &mut T) {
        if let Some(next_state) = self.redo_list.pop() {
            self.undo_list
                .push(std::mem::replace(current_state, next_state));
        }
    }

    /// Insert a new entry to the undo stack.
    /// NOTE: (IMP): If we have hit undo a few times then discard all the other values that come
    /// after the current point
    pub(super) fn insert(&mut self, current_state: T) {
        self.undo_list.push(current_state);
        self.redo_list.clear();
    }

    /// Reset the stack to the initial state
    pub(super) fn reset(&mut self) {
        self.undo_list.clear();
        self.redo_list.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    fn edit_stack<T>(undo_states: Vec<T>, redo_states: Vec<T>) -> EditStack<T>
    where
        T: Clone,
    {
        EditStack {
            undo_list: undo_states,
            redo_list: redo_states,
        }
    }

    #[rstest]
    #[case(edit_stack(vec![1, 2], vec![]), 3, 2)]
    #[case(edit_stack(vec![], vec![]), 1, 1)]
    fn undo_works(
        #[case] stack: EditStack<isize>,
        #[case] mut current: isize,
        #[case] value_after_undo: isize,
    ) {
        let mut stack = stack;

        stack.undo(&mut current);
        assert_eq!(current, value_after_undo);
    }

    #[rstest]
    #[case(edit_stack(vec![1], vec![3]), 2, 3)]
    #[case(edit_stack(vec![], vec![]), 1, 1)]
    fn redo_works(
        #[case] stack: EditStack<isize>,
        #[case] mut current: isize,
        #[case] value_after_undo: isize,
    ) {
        let mut stack = stack;

        stack.redo(&mut current);
        assert_eq!(current, value_after_undo);
    }

    #[rstest]
    #[case(edit_stack(vec![1, 2], vec![3]), 4, edit_stack(vec![1, 2, 4], vec![]))]
    #[case(edit_stack(vec![1, 2, 3], vec![]), 3, edit_stack(vec![1, 2, 3, 3], vec![]))]
    fn insert_works(
        #[case] old_stack: EditStack<isize>,
        #[case] value_to_insert: isize,
        #[case] expected_stack: EditStack<isize>,
    ) {
        let mut stack = old_stack;

        stack.insert(value_to_insert);
        assert_eq!(stack, expected_stack);
    }
}
