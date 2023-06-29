//! A memory storage comparable to a Vec where removing items doesn't shift all the items after the removed item to the left and doesn't invalidate their IDs. It allows you to remove items with high speed and access items via an ID that was returned after adding them.
//! # Example with array
//! ```
//! use memory_storage::new_with_array;
//!
//! let mut memory_storage = new_with_array::<i32, 3>();
//!
//! let id_of_one = memory_storage.insert(1)
//!     .expect("Something went wrong!");
//! let id_of_two = memory_storage.insert(2)
//!     .expect("Something went wrong!");
//! let id_of_three = memory_storage.insert(3)
//!     .expect("Something went wrong!");
//!
//! // We are at max capacity!
//! assert!(memory_storage.insert(4).is_err());
//!
//! let two = memory_storage.remove(id_of_two);
//! let id_of_four = memory_storage.insert(4)
//!     .expect("Something went wrong!");
//! let three = *memory_storage.get(id_of_three)
//!     .expect("Something went wrong!");
//!
//! assert_eq!(three, 3);
//! ```
//! # Example with vec (only with the 'alloc' feature)
//! ```
//! // Only with 'alloc' feature on!
//! use memory_storage::vec::new_with_fixed_capacity_vec;
//! use memory_storage::vec::new_with_vec;
//!
//! // Create a MemoryStorage using a vec with a fixed size of 3.
//! let fixed_size_vec_memory_storage = new_with_fixed_capacity_vec(3);
//!
//! // MemoryStorage using a vec allowing to allocate more space.
//! // Here we create an instance with the size of 1 (which can be increased).
//! let mut vec_memory_storage = new_with_vec(1);
//!
//! let id_of_one = vec_memory_storage.push(1);
//! let id_of_two = vec_memory_storage.push(2);
//! let id_of_three = vec_memory_storage.push(3);
//!
//! let three = *vec_memory_storage.get(id_of_three)
//!      .expect("Something went wrong!");
//!
//! assert_eq!(three, 3);
//! ```

#![no_std]

extern crate alloc;
extern crate core;

#[cfg(feature = "alloc")]
pub mod vec;

pub mod slot;

use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
use crate::slot::Slot;

/// The ID used to gain access to stored items.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Id(usize);

/// The instance that will store all the items.
pub struct MemoryStorage<T, U>
    where
        U: AsRef<[Slot<T>]> + AsMut<[Slot<T>]>, {
    pub(crate) storage: U,
    pub(crate) next_free_slot: Option<usize>,
    pub(crate) last_free_slot: Option<usize>,
    pub(crate) taken_slots: usize,
    pub(crate) capacity: usize,
    pub(crate) _marker: PhantomData<T>,
}

impl<T, U> MemoryStorage<T, U>
    where
        U: AsRef<[Slot<T>]> + AsMut<[Slot<T>]>, {
    pub fn clear(&mut self) {
        let capacity = self.capacity;
        if capacity == 0 {
            return;
        }
        for i in 0..capacity {
            self.storage
                .as_mut()[i] = Slot::NextFreeSlot(Some(i + 1));
        }
        self.storage
            .as_mut()[capacity - 1] = Slot::NextFreeSlot(None);
        self.next_free_slot = Some(0);
        self.last_free_slot = Some(capacity - 1);
        self.taken_slots = 0;
    }

    /// Try to insert an item. Returning the ID on a successful insert and returning the item wrapped in an error whenever there's no space left.
    pub fn insert(&mut self, item: T) -> Result<Id, InternalStorageFullError<T>> {
        match self.next_free_slot {
            None => Err(InternalStorageFullError(item)),
            Some(next_free_slot) => Ok(self.fill_free_slot(next_free_slot, item)),
        }
    }

    fn fill_free_slot(&mut self, free_slot: usize, item: T) -> Id {
        let next_free_slot = if let Slot::NextFreeSlot(next_free_slot) = self.storage.as_ref()[free_slot] {
            next_free_slot
        } else {
            unreachable!("Slot wasn't free!");
        };
        self.taken_slots += 1;
        match next_free_slot {
            None => {
                self.next_free_slot = None;
                self.last_free_slot = None;
            }
            Some(_) =>
                self.next_free_slot = next_free_slot,
        }
        self.storage.as_mut()[free_slot] = Slot::Taken(item);
        Id(free_slot)
    }

    /// Removes an item without shifting the items after it to the left and without invalidating their IDs.
    pub fn remove(&mut self, id: Id) -> T {
        let id = id.0;
        let slot = core::mem::replace(&mut self.storage.as_mut()[id], Slot::NextFreeSlot(None));
        if slot.is_free() {
            panic!("No item stored at index!");
        }
        self.taken_slots -= 1;
        match self.last_free_slot {
            Some(free_slot) => {
                if let Some(slot) = self.storage.as_mut().get_mut(free_slot) {
                    if let Slot::NextFreeSlot(next_free_slot) = slot {
                        next_free_slot.replace(id);
                    }
                } else {
                    unreachable!("Slot should exist!")
                }
                self.last_free_slot = Some(id);
            },
            None => {
                self.next_free_slot = Some(id);
                self.last_free_slot = Some(id);
            },
        }
        slot.unwrap_taken()
    }

    /// Returns a reference to an item whenever it is present.
    pub fn get(&self, id: Id) -> Option<&T> {
        self.storage
            .as_ref()
            .get(id.0)?
            .taken()
    }

    /// Returns a mutable reference to an item whenever it is present.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        self.storage
            .as_mut()
            .get_mut(id.0)?
            .taken_mut()
    }

    /// Returns the current capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns how many slots are currently in use.
    pub fn taken_slots(&self) -> usize {
        self.taken_slots
    }

    /// Acquire a reference to the storage.
    pub fn storage_ref(&self) -> &U {
        &self.storage
    }

    /// Destroy the instance and acquire the storage.
    pub fn storage(self) -> U {
        self.storage
    }
}

/// Alias for an array containing slots.
pub type SlotArray<T, const S: usize> = [Slot<T>; S];

/// Create a MemoryStorage instance using an array as storage.
pub fn new_with_array<T, const S: usize>() -> MemoryStorage<T, SlotArray<T, S>> {
    let array = initiate_array::<T, S>();
    let next_free_slot;
    let last_free_slot;
    if S == 0 {
        next_free_slot = None;
        last_free_slot = None;
    } else {
        next_free_slot = Some(0);
        last_free_slot = Some(S - 1);
    }
    MemoryStorage {
        storage: array,
        next_free_slot,
        last_free_slot,
        taken_slots: 0,
        capacity: S,
        _marker: Default::default(),
    }
}

fn initiate_array<T, const S: usize>() -> SlotArray<T, S> {
    let mut array: [Slot<T>; S] = core::array::from_fn(|i| {
        Slot::NextFreeSlot(Some(i + 1))
    });
    if S != 0 {
        array[S - 1] = Slot::NextFreeSlot(None);
    }
    array
}

/// Error to signal that the storage is full.
pub struct InternalStorageFullError<T>(pub T);

impl<T> InternalStorageFullError<T> {
    pub fn value(self) -> T {
        self.0
    }
}

impl<T> Debug for InternalStorageFullError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Internal storage is full!")
    }
}

impl<T> Display for InternalStorageFullError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{initiate_array, new_with_array};

    #[test]
    fn test_array() {
        let mut ms = new_with_array::<(), 3>();
        let _ = ms.insert(());
        let id = ms.insert(()).expect("I need this ID!");
        let _ = ms.insert(());
        ms.remove(id);
        let _ = ms.insert(());
        assert_eq!(ms.taken_slots, 3);
        ms.clear();
        assert_eq!(ms.taken_slots, 0);
    }

    #[test]
    fn test_initiate_array() {
        let array = initiate_array::<(), 3>();
        assert_eq!(array[0].next_free(), Some(1));
        assert_eq!(array[1].next_free(), Some(2));
        assert_eq!(array[2].next_free(), None);
    }
}