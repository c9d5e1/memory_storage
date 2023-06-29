# Memory Storage
A memory storage comparable to a Vec where removing items doesn't shift all the items after the removed item to the left and doesn't invalidate their IDs. It allows you to remove items with high speed and access items via an ID that was returned after adding them.
# Example with array
```
use memory_storage::new_with_array;

let mut memory_storage = new_with_array::<i32, 3>();

let id_of_one = memory_storage.insert(1)
    .expect("Something went wrong!");
let id_of_two = memory_storage.insert(2)
    .expect("Something went wrong!");
let id_of_three = memory_storage.insert(3)
    .expect("Something went wrong!");

// We are at max capacity!
assert!(memory_storage.insert(4).is_err());

let two = memory_storage.remove(id_of_two);
let id_of_four = memory_storage.insert(4)
    .expect("Something went wrong!");
let three = *memory_storage.get(id_of_three)
    .expect("Something went wrong!");

assert_eq!(three, 3);
```
# Example with vec (only with the 'alloc' feature)
```
// Only with 'alloc' feature on!
use memory_storage::vec::new_with_fixed_capacity_vec;
use memory_storage::vec::new_with_vec;

// Create a MemoryStorage using a vec with a fixed size of 3.
let fixed_size_vec_memory_storage = new_with_fixed_capacity_vec(3);

// MemoryStorage using a vec allowing to allocate more space.
// Here we create an instance with the size of 1 (which can be increased).
let mut vec_memory_storage = new_with_vec(1);

let id_of_one = vec_memory_storage.push(1);
let id_of_two = vec_memory_storage.push(2);
let id_of_three = vec_memory_storage.push(3);

let three = vec_memory_storage.get(id_of_three)
     .expect("Something went wrong!");

assert_eq!(three, 3);
```