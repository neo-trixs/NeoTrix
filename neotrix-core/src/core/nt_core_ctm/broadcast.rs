use super::chunk::Chunk;
use super::processor::LtmProcessor;

pub fn downtree_broadcast(processors: &mut [Box<dyn LtmProcessor>], winning: &Chunk) {
    for proc in processors.iter_mut() {
        proc.write_memory(winning);
    }
}
