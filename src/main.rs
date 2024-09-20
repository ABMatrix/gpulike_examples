use std::sync::{Arc, Mutex};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

type Block = [u8;512];
type HeaderEntry = [u8;512];

const CHUNKS_NUM: usize = 20;

fn fill_with_random_numbers(array: &mut [u8; 512]) {
    let mut rng = StdRng::from_entropy();
    rng.fill(array);
}

pub struct BlockEntry {
    pub block: Block,
    pub entry: HeaderEntry,
    pub size: u64,
}

#[derive(Clone)]
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

fn add_block(blockentries_ptr: Arc<Vec<BlockEntry>>, result_ptr: Vec<Arc<Mutex<Vec<BlockEntryB>>>>){
    let index = blockentries_ptr.len();

    let index: Vec<usize> = (0..index).into_iter().collect();

    let mut hs = Vec::new();

    for (i_chunks, i) in index.chunks(CHUNKS_NUM).enumerate(){

        let input_ptr = blockentries_ptr.clone();
        let each_result_ptr = result_ptr[i_chunks].clone();
        
        let start_index = i[0].clone();
        let offset = i.len();


        let h = std::thread::spawn( move ||{ 
            execution(input_ptr, each_result_ptr, start_index, offset);
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
        res_ptr[i-index].block = input_ptr[i].entry;  
        res_ptr[i-index].entry = input_ptr[i].block;        
        res_ptr[i-index].size = input_ptr[i].size;        
    }
}

fn main() {
    let blockentries = create_data(8000);
    let result = create_data_result(8000);

    // 输入指针是arc可以共同读
    let ptr_input = Arc::new(blockentries);

    // 预处理 result 的 指针
    let mut result_chunks = Vec::new();
    for each_res in result.chunks(CHUNKS_NUM){
        result_chunks.push(each_res.to_vec());
    }

    let result_chunks_to_be_modified_by_each_thread: Vec<Arc<Mutex<Vec<BlockEntryB>>>> = 
          result_chunks.into_iter().map(|t| Arc::new(Mutex::new(t))).collect();

    // 处理数据
    add_block(ptr_input.clone(), result_chunks_to_be_modified_by_each_thread.clone());

    // 还原结果
    let mut reorder_result = Vec::new();
    for each_res in result_chunks_to_be_modified_by_each_thread{
        reorder_result.append(&mut each_res.lock().unwrap());
    }

    let ptr_result = Arc::new(reorder_result);

    equal(ptr_input, ptr_result);
}



fn equal(blockentries_ptr: Arc<Vec<BlockEntry>>, result_ptr: Arc<Vec<BlockEntryB>>){
    let index = blockentries_ptr.len();

    let input_ptr = blockentries_ptr;
    let res_ptr = result_ptr;
    for i in 0..index {
        assert_eq!(res_ptr[i].block,input_ptr[i].entry);  
        assert_eq!(res_ptr[i].entry,input_ptr[i].block);        
        assert_eq!(res_ptr[i].size, input_ptr[i].size);    
    }
}