use core::{cell::UnsafeCell, mem};

use cortex_m::interrupt;

/// InterruptFreeCell is intended to be used in a static variable.
/// It synchronises accesses to the inner value by disabling interrupts
/// while accessing the inner value
///
/// # Safety
/// ** This Type should only be used on a single core system **
///
/// If the system has multiple cores, disabling interrupts is not
/// sufficient to synchronise access.
#[derive(Default, Debug)]
pub struct InterruptFreeCell<T> {
    value: UnsafeCell<T>,
}

// Safety:
// It is ok to allow access from multiple threads,
// because all accesses are synchronised internally by disabling interrupts.
unsafe impl<T> Sync for InterruptFreeCell<T> {}

impl<T> InterruptFreeCell<T> {
    pub const fn new(initial_value: T) -> Self {
        InterruptFreeCell {
            value: UnsafeCell::new(initial_value),
        }
    }

    /// Sets the content of the cell, and returns the old value
    #[must_use = "If the result of replace is not needed, use set instead"]
    pub fn replace(&self, new_value: T) -> T {
        interrupt::free(|_| {
            // Safety:
            // Accessing the value in the UnsafeCell is safe because no one else is accessing it:
            // The code is running on a single-threaded system, and interrupts have been disabled.
            let old_value = unsafe { &mut *self.value.get() };
            mem::replace(old_value, new_value)
        })
    }

    pub fn set(&self, new_value: T) {
        let _ = self.replace(new_value);
    }
}

impl<T> InterruptFreeCell<T>
where
    T: Default,
{
    pub fn take(&self) -> T {
        self.replace(Default::default())
    }
}

impl<T> InterruptFreeCell<T>
where
    T: Clone,
{
    pub fn get(&self) -> T {
        interrupt::free(|_| {
            // Safety:
            // Accessing the value in the UnsafeCell is safe because noone else is accessing it.
            // The code is running on a single-threaded system, and interrupts have been disabled.
            let old_value = unsafe { &mut *self.value.get() };
            old_value.clone()
        })
    }
}
