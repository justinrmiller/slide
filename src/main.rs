use dashmap::DashMap as DMF;
use std::collections::BinaryHeap;
use std::io;
use std::io::prelude::*;
use fxhash::FxBuildHasher;
use std::time::{Instant};
use std::cmp::Ordering;
use std::thread;
use ordered_float::OrderedFloat;

use evmap::{ReadHandle, WriteHandle};

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

type DashMap<K, V> = DMF<K, V, FxBuildHasher>;

const SIZE: u32 = 32 * 1024 * 1024;
const HEAP_CAPACITY: usize = 10;
const NTHREADS: u32 = 16;

#[derive(Clone, Hash)]
struct State {
    id: Box<Vec<u8>>,
    score: OrderedFloat<f32>,
}

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

fn generate_ev_map(heap_number: u32, size: u32) -> (ReadHandle<String, Box<State>>, WriteHandle<String, Box<State>>) {
    println!("Generating heap number: {} with size {}", heap_number, size);
    let mut rng = rand::thread_rng();
    let (r, mut w) = evmap::new();
    (0..size).for_each(|_| {
        let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
        let ranval = rng.gen_range(0.0, 1.0);
        let score = OrderedFloat(ranval);
        let blank = "".to_string();
        let a = blank.as_bytes();
        let b: Vec<u8> = a.iter().cloned().collect();
        w.insert(id, Box::new(State { id: Box::new(b), score: score } ));
    });
    w.refresh();
    return (r, w);
}

fn gen_ev_heap(readHandle: &ReadHandle<String, Box<State>>) -> BinaryHeap<State> {
    let mut heap = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);

    let now = Instant::now();

    for (key, value) in  &readHandle.read().unwrap() {
        match value.get_one() {
            None => println!("Oops get_one"),
            Some(val) => {
                let unboxedValue = &(*val);

                let keystring = key.to_string();
                let a = keystring.as_bytes();
                let b: Vec<u8> = a.iter().cloned().collect();
                heap.push(State { id: Box::new(b), score: unboxedValue.score });

                if heap.len() > HEAP_CAPACITY {
                    heap.pop();
                }
            },
        }        
    }

    println!("Size of heap: {}, Heap generation time: {}", heap.len(), now.elapsed().as_millis());
    return heap
}

fn main() {

    println!("Starting slide...");

    println!("Initializing memory with capacity {}...", SIZE);

    let mut maps = Box::new(Vec::new());
    for i in 0..NTHREADS {
        maps.push(generate_ev_map(i, SIZE / NTHREADS));
    }

    println!("Initializing memory with capacity {}...complete", SIZE);

    println!("Scanning with capacity {}...", SIZE);

    let now = Instant::now();

    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();

    for i in 0..NTHREADS {
        // Spin up another thread
        let (reader, _) = &maps[i as usize];
        let cloned_reader = reader.clone();

        let tx = tx.clone();
        thread::spawn(move || {
             let heap = gen_ev_heap(&cloned_reader);
             println!("Sending heap {}...", i);
             tx.send(heap).unwrap();
        });
    }

    let mut consolidated_heap = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);

    for i in 0..NTHREADS  {
        let received = rx.recv().unwrap();
        println!("Received {} items for heap {}", received.len(), i);
        for x in received {
            consolidated_heap.push(State { id: x.id, score: x.score });
            if consolidated_heap.len() > HEAP_CAPACITY {
                consolidated_heap.pop();
            }
        }
    }

    for x in consolidated_heap {
        println!("Response {} - {}", std::str::from_utf8(&*x.id).unwrap(), x.score);
    }




    println!("Scanning memory with capacity {}...complete", SIZE);
    println!("Took time: {} ms", now.elapsed().as_millis());

    println!("Displaying output");
    pause();
}

fn pause() {
    let mut stdout = io::stdout();

    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}