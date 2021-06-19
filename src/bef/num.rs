
pub trait Tone {
    fn value(&self) -> usize;
    fn from_usize(val: usize) -> Self where Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DynamicTone(usize);

impl DynamicTone {
    #[inline]
    pub const fn new(val: usize) -> DynamicTone {
        DynamicTone(val)
    }

    #[inline]
    pub fn from_boxed(tone: Box<dyn Tone>) -> DynamicTone {
        DynamicTone(tone.value())
    }
}

impl Tone for DynamicTone {
    fn value(&self) -> usize {
        self.0
    }

    fn from_usize(val: usize) -> Self where Self: Sized {
        Self(val)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct T1;

impl Tone for T1 {
    #[inline]
    fn value(&self) -> usize {
        1
    }

    #[inline]
    fn from_usize(val: usize) -> Self where Self: Sized {
        assert!(val == 1, "Mismatched tone.");
        T1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct T8;

impl Tone for T8 {
    #[inline]
    fn value(&self) -> usize {
        8
    }

    fn from_usize(val: usize) -> Self where Self: Sized {
        assert!(val == 8, "Mismatched tone.");
        T8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct T16;

impl Tone for T16 {
    #[inline]
    fn value(&self) -> usize {
        16
    }

    #[inline]
    fn from_usize(val: usize) -> Self where Self: Sized {
        assert!(val == 16, "Mismatched tone.");
        T16
    }
}