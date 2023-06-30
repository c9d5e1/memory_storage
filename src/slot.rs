use core::fmt::{Debug, Formatter};

/// A slot representing a spot in the storage.
pub enum Slot<T> {
    Taken(T),
    NextFreeSlot(Option<usize>),
}

impl<T> Slot<T> {
    pub fn is_taken(&self) -> bool {
        matches!(*self, Slot::Taken(_))
    }

    pub fn is_free(&self) -> bool {
        !self.is_taken()
    }

    pub fn taken(&self) -> Option<&T> {
        if let Slot::Taken(item) = self {
            Some(item)
        } else {
            None
        }
    }

    pub fn taken_mut(&mut self) -> Option<&mut T> {
        if let Slot::Taken(item) = self {
            Some(item)
        } else {
            None
        }
    }

    pub fn unwrap_taken(self) -> T {
        if let Slot::Taken(kill_switch) = self {
            kill_switch
        } else {
            panic!("Slot wasn't taken!")
        }
    }

    pub fn unwrap_next_free(self) -> Option<usize> {
        if let Slot::NextFreeSlot(next_free_slot) = self {
            next_free_slot
        } else {
            panic!("Slot wasn't free!")
        }
    }

    pub fn next_free(&self) -> Option<usize> {
        if let Slot::NextFreeSlot(next_free_slot) = self {
            *next_free_slot
        } else {
            panic!("Slot wasn't free!")
        }
    }
}

impl<T: Debug> Debug for Slot<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Slot::Taken(item) =>
                write!(f, "Slot::Taken({:#?})", item),
            Slot::NextFreeSlot(next_free_slot) =>
                write!(f, "Slot::NextFreeSlot({:#?})", next_free_slot),
        }
    }
}