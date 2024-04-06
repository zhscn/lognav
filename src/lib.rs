#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Position {
    row: usize,
    column: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Chunk {
    data: Vec<u8>,
    line_starts: Vec<usize>,
}

impl Chunk {
    fn new(data: Vec<u8>) -> Chunk {
        let mut line_starts =
            data.iter()
                .enumerate()
                .fold(vec![0], |mut line_starts, (i, &byte)| {
                    if byte == b'\n' {
                        line_starts.push(i + 1);
                    }
                    line_starts
                });
        if *line_starts.last().unwrap() != data.len() {
            line_starts.push(data.len());
        }
        Self { data, line_starts }
    }

    fn load(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    fn relase(&mut self) {
        self.data.clear();
    }

    fn get_line_count(&self) -> usize {
        self.line_starts.len() - 1
    }

    fn get_line_content(&self, idx: usize) -> Option<&[u8]> {
        if self.data.is_empty() || idx >= self.line_starts.len() - 1 {
            return None;
        }
        let start = *self.line_starts.get(idx).unwrap();
        let end = *self.line_starts.get(idx + 1).unwrap_or(&(self.data.len()));
        Some(&self.data[start..end])
    }

    fn continue_to_next_chunk(&self) -> bool {
        if self.data.is_empty() {
            return false;
        } else {
            *self.data.last().unwrap() != b'\n'
        }
    }

    fn calc_end(&self, start: Position) -> Position {
        let mut end = start;
        if self.data.is_empty() {
            return end;
        }

        let lines = &self.line_starts[..self.line_starts.len() - 1];
        end.row += lines.len() - 1;
        if end.row != start.row {
            end.column = 0;
        }
        end.column += self.data.len() - *lines.last().unwrap();
        if !self.continue_to_next_chunk() {
            end.row += 1;
            end.column = 0;
        }
        end
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn str_to_chunk(s: &str) -> Chunk {
        Chunk::new(s.as_bytes().to_vec())
    }

    #[test]
    fn test_new_chunk() {
        let start = Position { row: 0, column: 0 };
        let chunk = str_to_chunk("");
        assert_eq!(chunk.get_line_count(), 0);
        assert_eq!(chunk.line_starts, vec![0]);
        assert_eq!(chunk.get_line_content(0), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), start);
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), start);
        }

        let chunk = str_to_chunk("a");
        assert_eq!(chunk.get_line_count(), 1);
        assert_eq!(chunk.line_starts, vec![0, 1]);
        assert_eq!(chunk.get_line_content(0), Some(b"a".as_slice()));
        assert_eq!(chunk.continue_to_next_chunk(), true);
        assert_eq!(chunk.calc_end(start), Position { row: 0, column: 1 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 1, column: 2 });
        }

        let chunk = str_to_chunk("\n");
        assert_eq!(chunk.get_line_count(), 1);
        assert_eq!(chunk.line_starts, vec![0, 1]);
        assert_eq!(chunk.get_line_content(0), Some(b"\n".as_slice()));
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 1, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        }

        let chunk = str_to_chunk("\n\n");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_starts, vec![0, 1, 2]);
        assert_eq!(chunk.get_line_content(0), Some(b"\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"\n".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 3, column: 0 });
        }

        let chunk = str_to_chunk("a\n");
        assert_eq!(chunk.get_line_count(), 1);
        assert_eq!(chunk.line_starts, vec![0, 2]);
        assert_eq!(chunk.get_line_content(0), Some(b"a\n".as_slice()));
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 1, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        }

        let chunk = str_to_chunk("a\nb");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_starts, vec![0, 2, 3]);
        assert_eq!(chunk.get_line_content(0), Some(b"a\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"b".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), true);
        assert_eq!(chunk.calc_end(start), Position { row: 1, column: 1 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 2, column: 1 });
        }

        let chunk = str_to_chunk("a\nb\n");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_starts, vec![0, 2, 4]);
        assert_eq!(chunk.get_line_content(0), Some(b"a\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"b\n".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 3, column: 0 });
        }

        let mut c = chunk.clone();
        c.relase();
        assert_eq!(c.data, vec![]);
        c.load(chunk.data.clone());
        assert_eq!(c.data, chunk.data);
    }
}
