use sourcefile::SourceFile;

pub struct Tree {}

impl Iterator for Tree {
    type Item = SourceFile;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}