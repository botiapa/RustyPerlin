pub struct World {
    pub chunks: Vec<Chunk>
}

pub struct Chunk {
    pub blocks: Vec<Block>
}

pub struct Block {
    pub location: Location,
    pub color: [f32; 4]
}

pub struct Location {
    pub x: f64,
    pub y: f64,
}