use sourcefile::Sourcefile;

pub struct Tree {}

impl Iterator for Tree {
    type Item = Sourcefile;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}