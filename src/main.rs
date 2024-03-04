use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::SystemTime;

use rand::Rng;
use num_cpus;

fn main() {
    let now = SystemTime::now();
    let vec_length = 100_000;
    let vec_max = 10_000;
    let array_count = 100;

    for _i in 0..array_count {
        let mut random_vec = get_random_vec( vec_length, vec_max );
        quick_sort_par( &mut random_vec );
    }

    println!( "parallel qs average time: {}ms", now.elapsed().unwrap().as_millis() / array_count );

    let now = SystemTime::now();

    for _i in 0..array_count {
        let mut random_vec = get_random_vec( vec_length, vec_max );
        quick_sort( &mut random_vec );
    }

    println!( "sequential qs average time: {}ms", now.elapsed().unwrap().as_millis() / array_count );
}

fn quick_sort_par( a: &mut Vec<i32> ) {
    quick_sort_rec_par( a, 0, ( a.len() - 1 ) as i32, num_cpus::get() as i32 / 2 -1 );
}

fn quick_sort_rec_par( a: &mut Vec<i32>, low: i32, high: i32, depth: i32 ) {
    if low < high {
        let p: i32 = partition( a, low as usize, high as usize );

        let arc_one = Arc::new( Mutex::new( ( a.clone(), low, high, p, depth ) ) );
        let arc_two = Arc::clone( &arc_one );

        let ( tx_one, rx_one ) = mpsc::channel();
        let ( tx_two, rx_two ) = mpsc::channel();

        thread::spawn( move || {
            let touple = arc_one.lock().unwrap();
            let mut a = touple.0.clone();
            let low = touple.1;
            let p = touple.3;
            let depth = touple.4;

            drop( touple );

            if depth <= 1 {
                quick_sort_rec( &mut a, low, p - 1 );
            } else {
                quick_sort_rec_par( &mut a, low, p - 1, depth - 1 );
            }

            tx_one.send( a ).unwrap();
        } );

        thread::spawn( move || {
            let touple = arc_two.lock().unwrap();
            let mut a = touple.0.clone();
            let high = touple.2;
            let p = touple.3;
            let depth = touple.4;

            drop( touple );

            if depth <= 1 {
                quick_sort_rec( &mut a, p + 1, high );
            } else {
                quick_sort_rec_par( &mut a, p + 1, high , depth - 1 );
            }

            tx_two.send( a ).unwrap();
        } );

        let a_one = rx_one.recv().unwrap();
        let a_two = rx_two.recv().unwrap();

        for i in low..p {
            a[i as usize] = a_one[i as usize];
        }

        for i in p + 1..high + 1 {
            a[i as usize] = a_two[i as usize];
        }
    }
}

fn quick_sort( a: &mut Vec<i32> ) {
    quick_sort_rec( a, 0, ( a.len() - 1 ) as i32 );
}

fn quick_sort_rec( a: &mut Vec<i32>, low: i32, high: i32 ) {
    if low < high {
        let p = partition( a, low as usize, high as usize );
        quick_sort_rec( a, low, p - 1 );
        quick_sort_rec( a, p + 1, high );
    }
}

fn partition( a: &mut Vec<i32>, low: usize, high: usize ) -> i32 {
    let pivot = a[high];
    let mut i = low;

    for j in low..high {
        if a[j] < pivot {
            let temp = a[i];
            a[i] = a[j];
            a[j] = temp;
            i += 1;
        }
    }

    let temp = a[i];
    a[i] = a[high];
    a[high] = temp;
    i as i32
}

fn get_random_vec( len: i32, max: i32 ) -> Vec<i32> {
    let mut vec: Vec<i32> = Vec::with_capacity( len as usize );

    for _ in 0..len {
        vec.push( rand::thread_rng().gen_range(0.. max) );
    }

    return vec;
}