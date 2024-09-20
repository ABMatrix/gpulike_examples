use std::sync::{Arc, Mutex};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

type Block = [u8;512];
type HeaderEntry = [u8;512];

fn fill_with_random_numbers(array: &mut [u8; 512]) {
    let mut rng = StdRng::from_entropy();
    rng.fill(array);
}

pub struct BlockEntry {
    pub block: Block,
    pub entry: HeaderEntry,
    pub size: u64,
}

pub struct BlockEntryB {
    pub block: Block,
    pub entry: HeaderEntry,
    pub size: u64,
}

fn create_data(num: u64) -> Vec<BlockEntry> {
    (0..num).into_iter().map(|i|{
        let mut a = [0;512];
        let mut b = [0;512];
        fill_with_random_numbers(&mut a);
        fill_with_random_numbers(&mut b);

        BlockEntry{
            block: a,
            entry: b,
            size: i ,
        }
    }).collect()
}

fn create_data_result(num: u64) -> Vec<BlockEntryB> {
    (0..num).into_iter().map(|i|{
        let a = [0;512];
        let b = [0;512];

        BlockEntryB{
            block: a,
            entry: b,
            size: i ,
        }
    }).collect()
}

fn add_block(blockentries_ptr: Arc<Vec<BlockEntry>>, result_ptr: Arc<Mutex<Vec<BlockEntryB>>>){
    let index = blockentries_ptr.len();

    let index: Vec<usize> = (0..index).into_iter().collect();


    let mut hs = Vec::new();

    for i in index.chunks(20){
        let ptr = blockentries_ptr.clone();
        let result_ptr = result_ptr.clone();
        let ind = i[0].clone();
        let offset = i.len();

        let h = std::thread::spawn( move ||{ 
            execution(ptr, result_ptr, ind, offset);
        });

        hs.push(h);

    }
    hs.into_iter().for_each(|h| h.join().unwrap());

}

fn execution(ptr: Arc<Vec<BlockEntry>>, result_ptr: Arc<Mutex<Vec<BlockEntryB>>>,
            index: usize, offset: usize) {

    let input_ptr = ptr;
    let mut res_ptr = result_ptr.lock().unwrap();

    for i in index..index+offset{
        res_ptr[i].block = input_ptr[i].entry;  
        res_ptr[i].entry = input_ptr[i].block;        
        res_ptr[i].size = input_ptr[i].size;        
    }

}

fn main() {
    let blockentries = create_data(40000);
    let result = create_data_result(40000);

    let ptr_input = Arc::new(blockentries);
    let ptr_result = Arc::new(Mutex::new(result));

    add_block(ptr_input.clone(), ptr_result.clone());

    equal(ptr_input, ptr_result);
}



fn equal(blockentries_ptr: Arc<Vec<BlockEntry>>, result_ptr: Arc<Mutex<Vec<BlockEntryB>>>){
    let index = blockentries_ptr.len();

    let input_ptr = blockentries_ptr;
    let res_ptr = result_ptr.lock().unwrap();
    for i in 0..index {
        assert_eq!(res_ptr[i].block,input_ptr[i].entry);  
        assert_eq!(res_ptr[i].entry,input_ptr[i].block);        
        assert_eq!(res_ptr[i].size, input_ptr[i].size);    
    }
}