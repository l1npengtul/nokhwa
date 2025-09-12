use core::iter::FusedIterator;

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum DuplicateConstState {
    #[default]
    Started,
    EmitDupe,
    EmitReal,
    Finished
}

/// Duplicates the items in the iterator `MULTIPLIER` times.
///
/// 0 acts as 1.
///
/// This iterator is _fused_.
pub struct DuplicateConst<I, const MULTIPLIER: usize> where
    I: Iterator,
    <I as Iterator>::Item: Clone {
    iter: I,
    last_iter_item: Option<I::Item>,
    running_count: usize,
    state: DuplicateConstState,
}

impl<I, const MULTIPLIER: usize> DuplicateConst<I, MULTIPLIER> where I: Iterator,
                                                                     <I as Iterator>::Item: Clone {
    pub fn new(iter: I) -> DuplicateConst<I, MULTIPLIER> {
        DuplicateConst {
            iter,
            last_iter_item: None,
            running_count: MULTIPLIER.saturating_sub(1),
            state: DuplicateConstState::default(),
        }
    }
}

impl<I, const MULTIPLIER: usize> Iterator for DuplicateConst<I, MULTIPLIER> where
    I: Iterator,
    <I as Iterator>::Item: Clone
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            DuplicateConstState::Started => {
                match self.iter.next() {
                    Some(i) => {
                        if MULTIPLIER <= 1 {
                            self.state = DuplicateConstState::EmitReal;
                        } else {
                            self.state = DuplicateConstState::EmitDupe;
                        }
                        self.last_iter_item = Some(i.clone());
                        Some(i)
                    }
                    None => {
                        self.state = DuplicateConstState::Finished;
                        None
                    }
                }
            }
            DuplicateConstState::EmitDupe => {
                self.running_count = self.running_count.saturating_sub(1);
                if self.running_count <= 0 {
                    self.state = DuplicateConstState::EmitReal;
                }
                self.last_iter_item.clone()
            }
            DuplicateConstState::EmitReal => {
                match self.iter.next() {
                    Some(i) => {
                        self.last_iter_item = Some(i.clone());
                        self.running_count = MULTIPLIER.saturating_sub(1);
                        if MULTIPLIER <= 1 {
                            self.state = DuplicateConstState::EmitReal;
                        } else {
                            self.state = DuplicateConstState::EmitDupe;
                        }
                        Some(i)
                    }
                    None => {
                        self.state = DuplicateConstState::Finished;
                        None
                    }
                }
            }
            DuplicateConstState::Finished => {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let multi = if MULTIPLIER == 0 { 1 } else { MULTIPLIER + 1 };
        (lower * multi, upper.map(|u| u * multi))
    }
}

impl<I, const MULTIPLIER: usize> FusedIterator for DuplicateConst<I, MULTIPLIER> where I: Iterator, <I as Iterator>::Item: Clone {}

pub trait IterDuplicateConst: Iterator {
    fn duplicate_const<const MULTIPLIER: usize>(self) -> DuplicateConst<Self, MULTIPLIER> where Self: Sized, Self::Item: Clone {
        DuplicateConst::new(self)
    }
}

impl<I: ?Sized> IterDuplicateConst for I where I: Iterator {}

#[cfg(test)]
mod test {
    use crate::duplicate::IterDuplicateConst;

    #[test]
    pub fn zero_acts_as_one() {
        let initial = vec![0, 0, 0, 0];
        let test_condition = vec![0, 0, 0, 0];
        let result = initial.into_iter().duplicate_const::<0>().collect::<Vec<i32>>();
        assert_eq!(test_condition, result)
    }

    #[test]
    pub fn one_is_one() {
        let initial = vec![0, 0, 0, 0];
        let test_condition = vec![0, 0, 0, 0];
        let result = initial.into_iter().duplicate_const::<1>().collect::<Vec<i32>>();
        assert_eq!(test_condition, result)
    }

    #[test]
    pub fn multiples() {
        let initial = vec![0, 0, 0, 0, 0];
        let test_condition = vec![0; 25];
        let result = initial.into_iter().duplicate_const::<5>().collect::<Vec<i32>>();
        assert_eq!(test_condition, result)
    }
}
