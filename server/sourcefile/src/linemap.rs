use tower_lsp::lsp_types::Position;

pub struct LineMap {
    positions: Vec<usize>,
}

impl LineMap {
    pub fn new(source: &str) -> Self {
        let mut positions = vec![0];
        for (i, char) in source.char_indices() {
            if char == '\n' {
                positions.push(i + 1);
            }
        }

        LineMap { positions }
    }

    pub fn offset_for_position(&self, position: Position) -> usize {
        self.positions[position.line as usize] + (position.character as usize)
    }

    pub fn line_range_for_position(&self, position: Position) -> (usize, Option<usize>) {
        if (position.line + 1) as usize >= self.positions.len() {
            return (self.positions[position.line as usize], None)
        }
        (self.positions[position.line as usize], Some(self.positions[(position.line + 1) as usize] - 1))
    }
}

#[cfg(test)]
mod test {
    use tower_lsp::lsp_types::Position;

    use crate::linemap::LineMap;

    #[test]
    #[logging::scope]
    fn test_linemap() {
        struct Test {
            string: &'static str,
            pos: Position,
            offset: usize,
        }

        let cases = vec![
            Test {
                string: "sample\ntext",
                pos: Position { line: 1, character: 2 },
                offset: 9,
            },
            Test {
                string: "banana",
                pos: Position { line: 0, character: 0 },
                offset: 0,
            },
            Test {
                string: "banana",
                pos: Position { line: 0, character: 1 },
                offset: 1,
            },
            Test {
                string: "sample\ntext",
                pos: Position { line: 1, character: 0 },
                offset: 7,
            },
            Test {
                string: "sample\n\ttext",
                pos: Position { line: 1, character: 2 },
                offset: 9,
            },
            Test {
                string: "sample\r\ntext",
                pos: Position { line: 1, character: 0 },
                offset: 8,
            },
        ];

        for case in cases {
            let linemap = LineMap::new(case.string);

            let offset = linemap.offset_for_position(case.pos);

            assert_eq!(offset, case.offset, "{:?}", case.string);
        }
    }
}
