use alloc::vec::Vec;
use crate::{Id, MemoryStorage};
use core::convert::AsRef;
use core::convert::AsMut;
use crate::slot::Slot;

/// Alias for vector containing slots.
pub type SlotVec<T> = Vec<Slot<T>>;

impl<T> MemoryStorage<T, SlotVec<T>> {
    /// Push an item ignoring capacity limits. Once the max capacity has been reached the vec simply allocates more space.
    pub fn push(&mut self, item: T) -> Id {
        let item = match self.insert(item) {
            Ok(id) => return id,
            Err(err) => err.0,
        };
        // Allow the vec to allocate more space for itself by pushing at full capacity.
        self.storage
            .push(Slot::NextFreeSlot(None));
        // Remove the newly inserted value so that we can insert actual slots.
        self.storage.pop();
        let old_capacity = self.capacity;
        let new_capacity = self.storage
            .capacity();
        let (starting_index, slots_to_insert) = if old_capacity != 0 {
            (new_capacity - old_capacity, new_capacity - old_capacity)
        } else {
            (0, 1)
        };
        let mut next_free_index = starting_index;
        for _ in 0..slots_to_insert {
            next_free_index += 1;
            self.storage
                .push(Slot::NextFreeSlot(Some(next_free_index)));
        }
        // Make sure the last slot isn't pointing to a none existent slot.
        *self.storage
            .last_mut()
            .expect("This exists!'") = Slot::NextFreeSlot(None);
        match self.last_free_slot {
            None => {
                self.next_free_slot = Some(starting_index);
                self.last_free_slot = Some(new_capacity - 1);
            },
            Some(last_free_slot) => {
                *self.storage
                    .get_mut(last_free_slot)
                    .expect("This was the original last_free_slot!") = Slot::NextFreeSlot(Some(starting_index))
            },
        }
        self.capacity = new_capacity;
        self.insert(item)
            .expect("We just made space available!")
    }
}

/// Vector with a fixed size.
pub struct FixedCapacitySlotVec<T>(SlotVec<T>);

impl<T> AsRef<[Slot<T>]> for FixedCapacitySlotVec<T> {
    fn as_ref(&self) -> &[Slot<T>] {
        self.0
            .as_ref()
    }
}

impl<T> AsMut<[Slot<T>]> for FixedCapacitySlotVec<T> {
    fn as_mut(&mut self) -> &mut [Slot<T>] {
        self.0
            .as_mut()
    }
}

/// Create a MemoryStorage instance using a vec of a fixed size as storage.
pub fn new_with_fixed_capacity_vec<T>(capacity: usize) -> MemoryStorage<T, FixedCapacitySlotVec<T>> {
    let fixed_capacity_slot_vec = FixedCapacitySlotVec(initiate_vec(capacity));
    let next_free_slot;
    let last_free_slot;
    if capacity == 0 {
        next_free_slot = None;
        last_free_slot = None;
    } else {
        next_free_slot = Some(0);
        last_free_slot = Some(capacity - 1)
    }
    MemoryStorage {
        storage: fixed_capacity_slot_vec,
        next_free_slot,
        last_free_slot,
        taken_slots: 0,
        capacity,
        _marker: Default::default(),
    }
}

/// Create a MemoryStorage instance using a vec as storage.
pub fn new_with_vec<T>(capacity: usize) -> MemoryStorage<T, SlotVec<T>> {
    let vec = initiate_vec(capacity);
    let next_free_slot;
    let last_free_slot;
    if capacity == 0 {
        next_free_slot = None;
        last_free_slot = None;
    } else {
        next_free_slot = Some(0);
        last_free_slot = Some(capacity - 1)
    }
    MemoryStorage {
        storage: vec,
        next_free_slot,
        last_free_slot,
        taken_slots: 0,
        capacity,
        _marker: Default::default(),
    }
}

fn initiate_vec<T>(capacity: usize) -> SlotVec<T> {
    let mut vec = Vec::with_capacity(capacity);
    for i in 0..capacity {
        vec.push(Slot::NextFreeSlot(Some(i+1)));
    }
    if capacity != 0 {
        vec[capacity-1] = Slot::NextFreeSlot(None);
    }
    vec
}

#[cfg(test)]
mod tests {
    use crate::vec::{new_with_fixed_capacity_vec, new_with_vec};

    #[test]
    fn test_vec() {
        let mut ms = new_with_vec(3);
        let _ = ms.insert(());
        let id = ms.insert(()).expect("I need this ID!");
        let _ = ms.insert(());
        let _ = ms.push(());
        ms.remove(id);
        let _ = ms.insert(());
        let _ = ms.insert(());
        let _ = ms.insert(());
        assert_eq!(ms.capacity(), ms.storage.capacity());
        assert_eq!(ms.taken_slots(), 6);
    }

    #[test]
    fn test_fixed_vec() {
        let mut ms = new_with_fixed_capacity_vec(3);
        let _ = ms.insert(());
        let id = ms.insert(()).expect("I need this ID!");
        let _ = ms.insert(());
        ms.remove(id);
        let _ = ms.insert(());
        assert_eq!(ms.taken_slots, 3);
        ms.clear();
        assert_eq!(ms.taken_slots, 0);
    }
}