use std::sync::{Arc, Mutex};
use std::time::Instant;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

type Block = [u8; 512];
type HeaderEntry = [u8; 512];

const CHUNKS_NUM: usize = 160;

fn fill_with_random_numbers(array: &mut [u8; 512]) {
    let mut rng = StdRng::from_entropy();
    rng.fill(array);
}


#[derive(Clone)]
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
    (0..num)
        .into_iter()
        .map(|i| {
            let mut a = [0; 512];
            let mut b = [0; 512];
            fill_with_random_numbers(&mut a);
            fill_with_random_numbers(&mut b);

            BlockEntry {
                block: a,
                entry: b,
                size: i,
            }
        })
        .collect()
}

fn create_data_result(num: u64) -> Vec<BlockEntryB> {
    (0..num)
        .into_iter()
        .map(|i| {
            let a = [0; 512];
            let b = [0; 512];

            BlockEntryB {
                block: a,
                entry: b,
                size: i,
            }
        })
        .collect()
}

fn add_block(
    blockentries_ptr: Arc<Vec<BlockEntry>>,
    result_ptr: Vec<Arc<Mutex<Vec<BlockEntryB>>>>,
) {
    let index = blockentries_ptr.len();

    let index: Vec<usize> = (0..index).into_iter().collect();

    let mut hs = Vec::new();

    for (i_chunks, i) in index.chunks(CHUNKS_NUM).enumerate() {
        let input_ptr = blockentries_ptr.clone();
        let each_result_ptr = result_ptr[i_chunks].clone();

        let start_index = i[0].clone();
        let offset = i.len();

        let h = std::thread::spawn(move || {
            execution(input_ptr, each_result_ptr, start_index, offset);
        });

        hs.push(h);
    }
    hs.into_iter().for_each(|h| h.join().unwrap());
}

fn execution(
    ptr: Arc<Vec<BlockEntry>>,
    result_ptr: Arc<Mutex<Vec<BlockEntryB>>>,
    index: usize,
    offset: usize,
) {
    let input_ptr = ptr;
    let mut res_ptr = result_ptr.lock().unwrap();

    for i in index..index + offset {
        res_ptr[i - index].block = input_ptr[i].entry;
        res_ptr[i - index].entry = input_ptr[i].block;
        res_ptr[i - index].size = input_ptr[i].size;
    }
}


fn add_block_single_thread(blockentries: Vec<BlockEntry>){
    let mut res = Vec::new();
    for each in blockentries {
        res.push(BlockEntryB{
            block: each.entry,
            entry: each.block,
            size: each.size,
        });
    }
}

fn main() {
    let blockentries = create_data(160000);
    let result = create_data_result(160000);

    let start = Instant::now();
    add_block_single_thread(blockentries.clone());
    let duration = start.elapsed();
    // 打印执行时间
    println!("单线程 执行时间: {duration:?}");

    let start = Instant::now();

    // 输入指针是arc可以共同读
    let ptr_input = Arc::new(blockentries);

    // 预处理 result 的 指针, 把他分割成 Vec<Arc<Mutex<Vec<BlockEntryB>>>>
    let mut result_chunks_to_be_modified_by_each_thread = Vec::new();
    for each_res in result.chunks(CHUNKS_NUM) {
        result_chunks_to_be_modified_by_each_thread.push(Arc::new(Mutex::new(each_res.to_vec())));
    }
    let duration = start.elapsed();
    // 打印执行时间
    println!("gpu形态 线程数{CHUNKS_NUM} 数据预处理 执行时间: {duration:?}");

    let start = Instant::now();
    // 处理数据
    add_block(
        ptr_input.clone(),
        result_chunks_to_be_modified_by_each_thread.clone(),
    );

    let duration = start.elapsed();
    // 打印执行时间
    println!("gpu形态 线程数{CHUNKS_NUM} 执行时间 执行时间: {duration:?}");

    // 还原结果
    let mut reordered_result = Vec::new();
    for each_res in result_chunks_to_be_modified_by_each_thread {
        reordered_result.append(&mut each_res.lock().unwrap());
    }

    equal(ptr_input, Arc::new(reordered_result));
}

fn equal(blockentries_ptr: Arc<Vec<BlockEntry>>, result_ptr: Arc<Vec<BlockEntryB>>) {
    let index = blockentries_ptr.len();

    let input_ptr = blockentries_ptr;
    let res_ptr = result_ptr;
    for i in 0..index {
        assert_eq!(res_ptr[i].block, input_ptr[i].entry);
        assert_eq!(res_ptr[i].entry, input_ptr[i].block);
        assert_eq!(res_ptr[i].size, input_ptr[i].size);
    }
}
