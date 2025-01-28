pub enum Blob {
    Jpeg(Vec<u8>),
}

impl Blob {
    pub fn size(&self) -> usize {
        match self {
            Blob::Jpeg(vec) => vec.len(),
        }
    }
}
