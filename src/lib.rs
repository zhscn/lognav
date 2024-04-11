use anyhow::Result;
use std::{
    cmp::min,
    io::{Read, Seek},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Position {
    row: usize,
    column: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Chunk {
    data: Vec<u8>,
    line_start_offset: Vec<usize>,
    line_feed_offset: Vec<usize>,
}

impl Chunk {
    fn new(data: Vec<u8>) -> Chunk {
        let (mut line_starts, line_feed) = data.iter().enumerate().fold(
            (vec![0], vec![]),
            |(mut line_start, mut line_feed), (i, &byte)| {
                if byte == b'\n' {
                    line_feed.push(i);
                    line_start.push(i + 1);
                }
                (line_start, line_feed)
            },
        );
        if *line_starts.last().unwrap() != data.len() {
            line_starts.push(data.len());
        }
        Self {
            data,
            line_start_offset: line_starts,
            line_feed_offset: line_feed,
        }
    }

    fn get_line_count(&self) -> usize {
        let c = self.line_feed_offset.len();
        if self.continue_to_next_chunk() {
            c + 1
        } else {
            c
        }
    }

    fn get_line_content(&self, idx: usize) -> Option<&[u8]> {
        if self.data.is_empty() || idx >= self.get_line_count() {
            return None;
        }
        let start = *self.line_start_offset.get(idx).unwrap();
        let end = *self.line_start_offset.get(idx + 1).unwrap();
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

        let last_line_idx = self.get_line_count() - 1;

        end.row += last_line_idx;
        if end.row != start.row {
            end.column = 0;
        }

        end.column += self.get_line_content(last_line_idx).unwrap().len();

        if !self.continue_to_next_chunk() {
            end.row += 1;
            end.column = 0;
        }
        end
    }

    fn calc_backward_start(&self) -> Position {
        let mut pos = Position { row: 0, column: 0 };
        if self.data.is_empty() {
            return pos;
        }

        if self.continue_to_next_chunk() {
            pos.column += self
                .get_line_content(self.get_line_count() - 1)
                .unwrap()
                .len();
        }
        pos
    }

    fn calc_backward_end(&self, start: Position) -> Position {
        let mut end = start;
        if self.data.is_empty() {
            return end;
        }

        end.row += self.get_line_count() - 1;
        if !self.continue_to_next_chunk() {
            end.row += 1;
        }

        if end.row != start.row {
            end.column = 0;
            if *self.data.first().unwrap() != b'\n' {
                end.column = self.get_line_content(0).unwrap().len() - 1;
            }
        } else {
            end.column += self
                .get_line_content(self.get_line_count() - 1)
                .unwrap()
                .len();
        }

        end
    }
}

struct ChunkLoader<T> {
    reader: T,
    chunk_size: u64,
    total_size: u64,
}

impl<T: Seek + Read> ChunkLoader<T> {
    fn new(reader: T, chunk_size: u64, total_size: u64) -> Self {
        Self {
            reader,
            chunk_size,
            total_size,
        }
    }

    fn chunk_count(&self) -> u64 {
        (self.total_size + self.chunk_size - 1) / self.chunk_size
    }

    fn load_chunk(&mut self, idx: u64) -> Result<Chunk> {
        let offset = idx * self.chunk_size;
        let length = min(self.chunk_size, self.total_size - offset);
        self.reader.seek(std::io::SeekFrom::Start(offset as u64))?;
        let mut data = vec![0; length as usize];
        self.reader.read_exact(&mut data)?;
        Ok(Chunk::new(data))
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
        assert_eq!(chunk.line_start_offset, vec![0]);
        assert_eq!(chunk.get_line_content(0), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), start);
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), start);
        }
        assert_eq!(chunk.calc_backward_start(), start);
        assert_eq!(chunk.calc_backward_end(start), start);
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_backward_end(start), start);
        }

        let chunk = str_to_chunk("a");
        assert_eq!(chunk.get_line_count(), 1);
        assert_eq!(chunk.line_start_offset, vec![0, 1]);
        assert_eq!(chunk.get_line_content(0), Some(b"a".as_slice()));
        assert_eq!(chunk.continue_to_next_chunk(), true);
        assert_eq!(chunk.calc_end(start), Position { row: 0, column: 1 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 1, column: 2 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 1 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 0, column: 1 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 1, column: 2 }
            );
        }

        let chunk = str_to_chunk("\n");
        assert_eq!(chunk.get_line_count(), 1);
        assert_eq!(chunk.line_start_offset, vec![0, 1]);
        assert_eq!(chunk.get_line_content(0), Some(b"\n".as_slice()));
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 1, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 0 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 1, column: 0 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 2, column: 0 }
            );
        }

        let chunk = str_to_chunk("\n\n");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_start_offset, vec![0, 1, 2]);
        assert_eq!(chunk.get_line_content(0), Some(b"\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"\n".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 3, column: 0 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 0 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 2, column: 0 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 3, column: 0 }
            );
        }

        let chunk = str_to_chunk("a\n");
        assert_eq!(chunk.get_line_count(), 1);
        assert_eq!(chunk.line_start_offset, vec![0, 2]);
        assert_eq!(chunk.get_line_content(0), Some(b"a\n".as_slice()));
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 1, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 0 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 1, column: 1 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 2, column: 1 }
            );
        }

        let chunk = str_to_chunk("a\nb");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_start_offset, vec![0, 2, 3]);
        assert_eq!(chunk.get_line_content(0), Some(b"a\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"b".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), true);
        assert_eq!(chunk.calc_end(start), Position { row: 1, column: 1 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 2, column: 1 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 1 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 1, column: 1 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 2, column: 1 }
            );
        }

        let chunk = str_to_chunk("a\nb\n");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_start_offset, vec![0, 2, 4]);
        assert_eq!(chunk.get_line_content(0), Some(b"a\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"b\n".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 3, column: 0 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 0 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 2, column: 1 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 3, column: 1 }
            );
        }

        let chunk = str_to_chunk("\na\n");
        assert_eq!(chunk.get_line_count(), 2);
        assert_eq!(chunk.line_start_offset, vec![0, 1, 3]);
        assert_eq!(chunk.get_line_content(0), Some(b"\n".as_slice()));
        assert_eq!(chunk.get_line_content(1), Some(b"a\n".as_slice()));
        assert_eq!(chunk.get_line_content(2), None);
        assert_eq!(chunk.continue_to_next_chunk(), false);
        assert_eq!(chunk.calc_end(start), Position { row: 2, column: 0 });
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(chunk.calc_end(start), Position { row: 3, column: 0 });
        }
        assert_eq!(chunk.calc_backward_start(), Position { row: 0, column: 0 });
        assert_eq!(
            chunk.calc_backward_end(start),
            Position { row: 2, column: 0 }
        );
        {
            let start = Position { row: 1, column: 1 };
            assert_eq!(
                chunk.calc_backward_end(start),
                Position { row: 3, column: 0 }
            );
        }
    }
}
