use core::ops::AddAssign;

pub fn min_max_range<N: Copy + PartialOrd + AddAssign<N> + Sized>(min: N, max: N, step: N) -> Vec<N> {
    let mut counter = min;
    let mut nums = vec![min];

    loop {
        counter += step;

        if counter > max {
            break
        }

        nums.push(counter);
    }

    nums
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FailedMathOp;

pub(crate) trait FallibleDiv {
    type Output;

    type Error: Default;

    fn fallible_div(&self, other: &Self) -> Result<Self::Output, Self::Error>;
}

pub(crate) trait FallibleSub {
    type Output;

    type Error: Default;

    fn fallible_sub(&self, other: &Self) -> Result<Self::Output, Self::Error>;
}

