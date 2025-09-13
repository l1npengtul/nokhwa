use core::iter::FusedIterator;

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum InterweaveState {
    EmitFake,
    #[default]
    EmitReal,
    Finished
}

/// Interweaves a value for every `PER` values of the original iterator.
///
/// `emit_last`: Allows you to set if a last item should be emitted even though it is not the "turn"
/// of the "value" yet, e.g.
///
/// ```
/// let initial = vec![1, 2, 1, 2, 1];
//  let test_condition = vec![1, 2, 3, 1, 2, 3, 1, 3];
//  let result = initial.iter().interweave::<2>(&3, true).cloned().collect::<Vec<i32>>();
//  assert_eq!(test_condition, result);
/// ```
///
/// This iterator is _fused_.
pub struct Interweave<I, const PER: usize> where
    I: Iterator,
    <I as Iterator>::Item: Clone {
    element: I::Item,
    iter: I,
    prev_state: InterweaveState,
    state: InterweaveState,
    count: usize,
    emit_last: bool,
}

impl<I, const PER: usize> Interweave<I, PER> where I: Iterator, <I as Iterator>::Item: Clone {
    pub fn new(item: I::Item, iterator: I, emit_last: bool) -> Interweave<I, PER> {
        Interweave {
            element: item,
            iter: iterator,
            prev_state: InterweaveState::default(),
            state: InterweaveState::default(),
            count: PER,
            emit_last,
        }
    }
}

impl<I, const PER: usize> Iterator for Interweave<I, PER> where I: Iterator, <I as Iterator>::Item: Clone {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            InterweaveState::EmitFake => {
                self.count = PER;
                self.prev_state = InterweaveState::EmitFake;
                self.state = InterweaveState::EmitReal;
                Some(self.element.clone())
            }
            InterweaveState::EmitReal => {
                self.count = self.count.saturating_sub(1);
                if let Some(i) = self.iter.next() {
                    if self.count == 0 {
                        self.state = InterweaveState::EmitFake;
                    } else {
                        self.state = InterweaveState::EmitReal;
                    }
                    self.prev_state = InterweaveState::EmitReal;
                    Some(i)
                } else {
                    self.state = InterweaveState::Finished;
                    if self.emit_last && self.prev_state != InterweaveState::EmitFake {
                        Some(self.element.clone())
                    } else {
                        None
                    }
                }
            }
            InterweaveState::Finished => {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        if PER == 1 || PER == 0 {
            return (lower * 2, upper.map(|u| u * 2))
        }

        let last = usize::from(self.emit_last);
        let new_lower = lower + (lower / PER) + last;
        let new_upper = upper.map(|u| { u + (u / PER) + last });
        (new_lower, new_upper)
    }
}

impl<I, const PER: usize> FusedIterator for Interweave<I, PER> where I: Iterator, <I as Iterator>::Item: Clone {}

pub trait IterInterweave: Iterator {
    fn interweave<const PER: usize>(self, item: Self::Item, emit_last: bool) -> Interweave<Self, PER> where Self: Sized, Self::Item: Clone {
        Interweave::new(item, self, emit_last)
    }
}

impl<I: ?Sized> IterInterweave for I where I: Iterator {}

#[cfg(test)]
mod test {
    use crate::interweave::IterInterweave;

    #[test]
    pub fn zero_interweave() {
        let initial = vec![0, 0, 0, 0, 0];
        let test_condition = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        let result = initial.iter().interweave::<1>(&1, false).cloned().collect::<Vec<i32>>();
        assert_eq!(test_condition, result);
    }

    #[test]
    pub fn empty_interweave_no_last() {
        let initial: Vec<i32> = vec![];
        let test_condition: Vec<i32> = vec![];
        let result = initial.iter().interweave::<1>(&1, false).cloned().collect::<Vec<i32>>();
        assert_eq!(test_condition, result);
    }

    #[test]
    pub fn empty_interweave_with_last() {
        let initial: Vec<i32> = vec![];
        let test_condition: Vec<i32> = vec![1];
        let result = initial.iter().interweave::<1>(&1, true).cloned().collect::<Vec<i32>>();
        assert_eq!(test_condition, result);
    }

    #[test]
    pub fn interweave_every_other() {
        let initial = vec![1, 2, 1, 2, 1 ,2];
        let test_condition = vec![1, 2, 3, 1, 2, 3, 1 , 2, 3];
        let result = initial.iter().interweave::<2>(&3, true).cloned().collect::<Vec<i32>>();
        assert_eq!(test_condition, result);
    }

    #[test]
    pub fn interweave_every_other_not_fitting() {
        let initial = vec![1, 2, 1, 2, 1];
        let test_condition = vec![1, 2, 3, 1, 2, 3, 1, 3];
        let result = initial.iter().interweave::<2>(&3, true).cloned().collect::<Vec<i32>>();
        assert_eq!(test_condition, result);
    }
}

