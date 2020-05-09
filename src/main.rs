use dashmap::DashMap as DMF;
// use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io;
use std::io::prelude::*;
use fxhash::FxBuildHasher;
// use rayon::prelude::*;
use std::time::{Instant};
use std::cmp::Ordering;
// use std::collections::{BTreeSet};
use std::thread;
use ordered_float::OrderedFloat;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

type DashMap<K, V> = DMF<K, V, FxBuildHasher>;

const SIZE: u32 = 8 * 1024 * 1024;
const HEAP_CAPACITY: usize = 5;
const NTHREADS: usize = 1;
//const SIZE: u64 = 10;

#[derive(Clone, Hash)]
struct State {
    id: Box<Vec<u8>>,
    score: OrderedFloat<f32>,
    // packages: BTreeSet::<u32>
    // position: usize,
}

// use evmap::shallow_copy::ShallowCopy;
// use std::mem::ManuallyDrop;

// impl ShallowCopy for State {
//     unsafe fn shallow_copy(&self) ->    <Self> {
//         ManuallyDrop::new(*self)
//     }
// }

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.score == other.score
    }
}

impl Eq for State {}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.score.partial_cmp(&self.score).unwrap()
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// fn generate_map(size: u32, partitions: u32) -> DashMap<String, State> {
//     let mut rng = rand::thread_rng();
//     let map = DashMap::<String, State>::with_capacity_and_hasher(size as usize, FxBuildHasher::default());
//     (0..size / partitions).for_each(|_| {
//         let id: String = thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(10)
//         .collect();
//         let ranval = rng.gen_range(0.0, 1.0);
//         let score = OrderedFloat(ranval);
//         map.insert(id, State { id: "".to_string(), score: score, packages: [1,2,3,4,5,6,7,8,9,10].iter().cloned().collect() } );
//     });
//     return map;
// }

use evmap::ReadHandle;
use evmap::WriteHandle;
fn generate_ev_map(size: u32, partitions: u32) -> (ReadHandle<String, Box<State>>, WriteHandle<String, Box<State>>) {
    let mut rng = rand::thread_rng();
    let (r, mut w) = evmap::new();
    (0..size / partitions).for_each(|_| {
        let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
        let ranval = rng.gen_range(0.0, 1.0);
        let score = OrderedFloat(ranval);
        let blank = "".to_string();
        let a = blank.as_bytes();
        let b: Vec<u8> = a.iter().cloned().collect();
        w.insert(id, Box::new(State { id: Box::new(b), score: score, /*packages: [1,2,3,4,5,6,7,8,9,10].iter().cloned().collect()*/ } ));
    });
    w.refresh();
    return (r, w);
}

// fn gen_heap(map: &DashMap<String, State>) -> BinaryHeap<State> {
//     let mut heap = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);
//     let _map: &DashMap<String, State> = &map;

//     for (key, value) in *map {
//         if heap.len() > HEAP_CAPACITY {
//             heap.pop();
//         }
//         heap.push(State { id: key, score: value.score, packages: value.packages });
//     }
//     heap.pop();
//     return heap
// }

fn gen_ev_heap(readHandle: &ReadHandle<String, Box<State>>) -> BinaryHeap<State> {
    let mut heap = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);
    // let _handle: ReadHandle<String, Box<State>> = &map;

    for (key, value) in  &readHandle.read().unwrap() {
        match value.get_one() {
            None => println!("Oops get_one"),
            Some(val) => {
                let unboxedValue = &(*val);

                if heap.len() > HEAP_CAPACITY {
                    heap.pop();
                }
                let keystring = key.to_string();
                let a = keystring.as_bytes();
                let b: Vec<u8> = a.iter().cloned().collect();
                heap.push(State { id: Box::new(b), score: unboxedValue.score });
            },
        }        
    }
    heap.pop();

    println!("Size of heap: {}", heap.len());
    return heap
}

fn main() {

    println!("Starting slide...");

    println!("Initializing memory with capacity {}...", SIZE);

    let mut maps = Box::new(Vec::new());
    maps.push(generate_ev_map(SIZE, 2));
    maps.push(generate_ev_map(SIZE, 2));

    // (0..SIZE / 2).for_each(|i| {
    //     let id: String = thread_rng()
    //     .sample_iter(&Alphanumeric)
    //     .take(10)
    //     .collect();
    //     maps[0].insert(id, State { id: "".to_string(), score: rng.gen_range(0.0, 1.0), packages: [1,2,3,4,5,6,7,8,9,10].iter().cloned().collect() } );
    // });
    // (0..SIZE / 2).for_each(|i| {
    //     let id: String = thread_rng()
    //     .sample_iter(&Alphanumeric)
    //     .take(10)
    //     .collect();
    //     maps[1].insert(id, State { id: "".to_string(), score: rng.gen_range(0.0, 1.0), packages: [1,2,3,4,5,6,7,8,9,10].iter().cloned().collect() } );
    // });

    println!("Initializing memory with capacity {}...complete", SIZE);

    println!("Scanning with capacity {}...", SIZE);

    let now = Instant::now();

    // let mut heap1 = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);
    // let mut heap2 = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);

    // let mut heaps = Vec::new();
    // heaps.push(BinaryHeap::<State>::with_capacity(HEAP_CAPACITY));
    // heaps.push(BinaryHeap::<State>::with_capacity(HEAP_CAPACITY));

    //map.into_par_iter().for_each(|key, value| {

    // for (key, value) in map {
    //     if heap.len() > HEAP_CAPACITY {
    //         heap.pop();
    //     }
    //     heap.push(State { id: key, score: value.score, packages: value.packages });
    // }

    // let handle1 = thread::spawn(move || {
    //     for (key, value) in maps[0] {
    //         if heap1.len() > HEAP_CAPACITY {
    //             heap1.pop();
    //         }
    //         heap1.push(State { id: key, score: value.score, packages: value.packages });
    //     }

    //     heap1.pop();
    // });
    // let handle2 = thread::spawn(move || {
    //     for (key, value) in maps[1] {
    //         if heap2.len() > HEAP_CAPACITY {
    //             heap2.pop();
    //         }
    //         heap2.push(State { id: key, score: value.score, packages: value.packages });
    //     }
    //     heap2.pop();
    // });
    // handle1.join().unwrap();
    // handle2.join().unwrap();

    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();

    // let mut handles = vec![];

    for i in 0..NTHREADS {
        // Spin up another thread
        let (reader, _) = &maps[i];
        let cloned_reader = reader.clone();

        thread::spawn(move || {
             let heap = gen_ev_heap(&cloned_reader);
             tx.send(heap).unwrap();
        });
    }

    // thread::spawn(move || {
    //     let val = String::from("hi");
    //     tx.send(val).unwrap();
    // });

    // let received = rx.recv().unwrap();
    // println!("Got: {}", received);

    // for child in handles.iter() {
    //     // Wait for the thread to finish. Returns a result.
    //     let returned_heap = child.join();
    //     match returned_heap {
    //         Err(_) => println!("Oops returned_heaps"),
    //         Ok(heap) => {
    //             println!("Heap size: {}", heap.len());
    //             heaps.push(heap);
    //         },
    //     }
    // }
    use std::str;

    let received = rx.recv().unwrap();
    for x in received {
        println!("{} - {}", str::from_utf8(&x.id).unwrap(), x.score);
    }
    // for item in received {
    //     match item {
    //         Err(_) => println!("Oops returned_heaps"),
    //         Ok(heap) => {
    //             println!("Heap size: {}", heap.len());
    //         },
    //     }
    // }

    println!("Scanning memory with capacity {}...complete", SIZE);
    println!("Took time: {}", now.elapsed().as_millis());

    println!("Displaying output");
    // for i in 0..heaps.len() {
    //     for x in heaps[i].iter() {
    //         println!("{} - {}", x.id, x.score);
    //     }
    // }
    // for heap in heaps.iter() {
    //     println!("{} - {}", heap.id, heap.score);
    // }
    pause();
}

fn pause() {
    let mut stdout = io::stdout();

    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}