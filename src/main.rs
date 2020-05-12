use std::collections::BinaryHeap;
use std::io;
use std::io::prelude::*;
use std::time::{Instant};
use std::thread;
use ordered_float::OrderedFloat;

use evmap::{ReadHandle, WriteHandle};

use rand::{Rng};
use rand::distributions::Alphanumeric;

const SIZE: u32 = 8 * 1024 * 1024;
const HEAP_CAPACITY: usize = 10;
const NTHREADS: u32 = 8;

mod state;
use state::State;

fn generate_ev_map(heap_number: u32, size: u32) -> (ReadHandle<String, Box<State>>, WriteHandle<String, Box<State>>) {
    println!("Generating heap number: {} with size {}", heap_number, size);
    let (r, mut w) = evmap::new();
    let mut rng = rand::thread_rng();

    (0..size).for_each(|_| {
        let id: String = rng
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();

        // let ranval = rng.gen_range(0.0, 1.0);
        // let score = OrderedFloat(rng.gen_range(0.0, 1.0));
        // let blank = "".to_string();

        // let a = blank.as_bytes();
        // let b: Box<Vec<u8>> = Box::new(a.iter().cloned().collect());
        w.insert(id, Box::new(State {
            id: "".to_string(), 
            score: OrderedFloat(rng.gen_range(0.0, 1.0)),
            scorea: OrderedFloat(rng.gen_range(0.0, 1.0)),
            scoreb: OrderedFloat(rng.gen_range(0.0, 1.0)),
            scorec: OrderedFloat(rng.gen_range(0.0, 1.0)),
            scored: OrderedFloat(rng.gen_range(0.0, 1.0)),

        }));
    });
    w.refresh();
    return (r, w);
}

fn gen_ev_heap(read_handle: &ReadHandle<String, Box<State>>) -> BinaryHeap<State> {
    let mut heap = BinaryHeap::<State>::with_capacity(HEAP_CAPACITY);

    let now = Instant::now();

    for (key, value) in  &read_handle.read().unwrap() {
        match value.get_one() {
            None => println!("Oops get_one"),
            Some(val) => {
                heap.push(State { 
                    id: key.to_string(), 
                    score: OrderedFloat::from((*val).scorea.into_inner() + (*val).scoreb.into_inner() + (*val).scorec.into_inner() + (*val).scored.into_inner()),
                    scorea: (*val).scorea,
                    scoreb: (*val).scoreb,
                    scorec: (*val).scorec,
                    scored: (*val).scored,
                });

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
    // for i in 0..NTHREADS {
    //     maps.push(generate_ev_map(i, SIZE / NTHREADS));
    // }

    // for i in 0..NTHREADS {
    //     // Spin up another thread
    //    let (reader, _) = &maps[i as usize];

    //     thread::spawn(move || {
    //         &maps.push(generate_ev_map(i, SIZE / NTHREADS));
    //     });
    // }

    use std::sync::mpsc;
    
    let (tx_gen, rx_gen) = mpsc::channel();

    for i in 0..NTHREADS {
        // Spin up another thread
        let tx = tx_gen.clone();
        thread::spawn(move || {
            let map = generate_ev_map(i, SIZE / NTHREADS);
            println!("Sending map {}...", i);
            tx.send(map).unwrap();
        });
    }

    for i in 0..NTHREADS  {
        let received = rx_gen.recv().unwrap();
        println!("Received map number {}", i);
        maps.push(received);
    }

    println!("Initializing memory with capacity {}...complete", SIZE);

    println!("Scanning with capacity {}...", SIZE);

    let now = Instant::now();

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
            consolidated_heap.push(State { 
                id: x.id, 
                score: x.score,
                scorea: x.scorea,
                scoreb: x.scoreb,
                scorec: x.scorec,
                scored: x.scored,
                
            });
            if consolidated_heap.len() > HEAP_CAPACITY {
                consolidated_heap.pop();
            }
        }
    }

    for x in consolidated_heap {
        println!("Response {} - {} - {} - {} - {} - {}", 
            x.id,
            x.score,
            x.scorea,
            x.scoreb,
            x.scorec,
            x.scored,
        );
    }

    println!("Scanning memory with capacity {}...complete", SIZE);
    println!("Took time: {} ms", now.elapsed().as_millis());


    pause();
}

fn pause() {
    let mut stdout = io::stdout();

    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}