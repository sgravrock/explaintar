use std::io::{self, Read};
use std::string::FromUtf8Error;
#[cfg(test)]
use std::fs::File;

fn main() {
    let iter = EntryIterator::from_stream(io::stdin());
    let mut i = 0;

    for entry in iter {
        let header = entry.header;

        if i == 0 {
            if header.has_magic() {
                println!("This looks like a valid tar file.");
            } else {
                println!("Bad magic in the first header.");
                break;
            }
        }

        if header.is_null() {
            println!("Entry {} is null", i);
            break;
        }

        println!("Entry {}", i);

        match header.name() {
            Ok(n) => println!("Name {}", n),
            Err(e) => println!("Invalid name: {}", e),
        };

        println!("Size {} bytes", header.size());
        println!("");
        i += 1;
    }
}


struct Entry {
    header: Header,
}

struct Header {
    block: [u8; 512],
}

const NAME_LEN: usize = 100;
const MODE_LEN: usize = 8;
const UID_LEN: usize = 8;
const GID_LEN: usize = 8;
const SIZE_LEN: usize = 12;
const MTIME_LEN: usize = 12;
const CHECKSUM_LEN: usize = 8;
const TYPEFLAG_LEN: usize = 1;
const LINKNAME_LEN: usize = 100;
const MAGIC_LEN: usize = 6;

impl Header {
    fn from_block(block: [u8; 512]) -> Header {
        Header { block }
    }

    fn has_magic(&self) -> bool {
        let maybe_magic = String::from_utf8(self.magic_field().to_vec());

        match maybe_magic {
            Ok(actual) => actual == "ustar\0",
            _ => false,
        }
    }

    fn is_null(&self) -> bool {
        let offset = NAME_LEN + MODE_LEN + UID_LEN + GID_LEN + SIZE_LEN + MTIME_LEN + CHECKSUM_LEN;
        self.block[offset] == 0
    }

    fn name(&self) -> Result<String, FromUtf8Error> {
        let len = find_zero(&self.block, NAME_LEN).unwrap_or(NAME_LEN);
        let bytes = self.block[0..len].to_vec();
        String::from_utf8(bytes)
    }

    fn size(&self) -> usize {
        let bytes = self.size_field();
        parse_octal(&bytes[0..SIZE_LEN - 1]) // Ignore the terminating space.
    }

    fn magic_field(&self) -> &[u8] {
        let offset = NAME_LEN + MODE_LEN + UID_LEN + GID_LEN + SIZE_LEN + MTIME_LEN +
                     CHECKSUM_LEN + TYPEFLAG_LEN + LINKNAME_LEN;
        &self.block[offset..(offset + MAGIC_LEN)]
    }

    fn size_field(&self) -> &[u8] {
        let offset = NAME_LEN + MODE_LEN + UID_LEN + GID_LEN;
        &self.block[offset..(offset + SIZE_LEN)]
    }
}

// TODO: don't panic if this fails,
// or at least panic somewhere else.
fn parse_octal(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .fold(0, |acc, b| {
            let n = *b as usize;
            if n < 48 || n > 55 {
                panic!("Not an octal digit: {}", b);
            }
            acc * 8 + n - 48
        })
}

fn find_zero(buf: &[u8; 512], maxlen: usize) -> Option<usize> {
    buf.iter()
        .take(maxlen)
        .enumerate()
        .find(|&(_, &e)| e == 0)
        .map(|(i, _)| i)
}

#[test]
fn test_has_magic() {
    let good = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000000 13124523641 013414^@ 0^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@ustar");
    let bad = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000000 13124523641 013414^@ 0^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@nope!");
    assert_eq!(true, Header::from_block(good).has_magic());
    assert_eq!(false, Header::from_block(bad).has_magic());
}

#[test]
fn test_is_null() {
    let null_block = [0; 512];
    let non_null_block = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000000 13124523641 013414^@ 0^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@ustar");
    assert_eq!(true, Header::from_block(null_block).is_null());
    assert_eq!(false, Header::from_block(non_null_block).is_null());
}

#[test]
fn test_name_short() {
    let block = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@x");
    assert_eq!("somefile", Header::from_block(block).name().unwrap());
}

#[test]
fn test_name_exactly_100() {
    let block = block_from_visual("long________________________________________________________________________________________________x");
    assert_eq!("long________________________________________________________________________________________________",
               Header::from_block(block).name().unwrap());
}

#[test]
fn test_size_small() {
    let block = block_from_visual("11bytes^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000013 ");
    assert_eq!(11, Header::from_block(block).size());
}

#[cfg(test)]
fn block_from_visual(visual: &str) -> [u8; 512] {
    let mut block = [0; 512];
    let chars: Vec<char> = visual.chars().collect();
    let mut i = 0;
    let mut j = 0;

    while i < chars.len() {
        if chars[i] == '^' && chars[i + 1] == '@' {
            i += 2;
        } else {
            let mut buf: [u8; 1] = [0; 1];
            chars[i].encode_utf8(&mut buf);
            block[j] = buf[0];
            i += 1;
        }

        j += 1;
    }

    block
}

#[test]
fn test_block_from_visual() {
    let block = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000000 13124523641 013414^@ 0^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@ustar^@00pivotal^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@staff^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000000 ^@000000 ^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@X");
    assert_eq!(115, block[0]); // s ascii
    assert_eq!(101, block[7]); // e ascii
    assert_eq!(0, block[8]); // ^@ becomes 0
    assert_eq!(48, block[101]); // 0 ascii
    assert_eq!(88, block[511]);
}

fn num_data_blocks(entry_size: usize) -> usize {
    f32::ceil(entry_size as f32 / 512.0) as usize
}

#[test]
fn test_num_data_blocks() {
    assert_eq!(0, num_data_blocks(0));
    assert_eq!(1, num_data_blocks(1));
    assert_eq!(1, num_data_blocks(512));
    assert_eq!(2, num_data_blocks(513));
}


struct BlockIterator<T: Read> {
    stream: T,
}

impl<T: Read> BlockIterator<T> {
    fn from_stream(stream: T) -> BlockIterator<T> {
        BlockIterator { stream }
    }
}

impl<T: Read> Iterator for BlockIterator<T> {
    type Item = [u8; 512];

    fn next(&mut self) -> Option<[u8; 512]> {
        let mut block: [u8; 512] = [0; 512];
        match self.stream.read(&mut block) {
            Ok(512) => Some(block),
            Ok(0) => None,
            Ok(n) => panic!("Expected to read 512 bytes but got {}", n),
            Err(e) => panic!("Read error: {}", e),
        }
    }
}

#[test]
fn test_block_iterator() {
    let file = File::open("fixtures/simple.tar").unwrap();
    let subject = BlockIterator::from_stream(file);
    let blocks: Vec<[u8; 512]> = subject.collect();
    assert_eq!(7, blocks.len());
}


struct EntryIterator<T: Read> {
    iter: BlockIterator<T>,
    done: bool,
}

impl<T: Read> EntryIterator<T> {
    fn from_stream(stream: T) -> EntryIterator<T> {
        EntryIterator {
            iter: BlockIterator::from_stream(stream),
            done: false,
        }
    }

    fn _make_entry(&mut self, block: [u8; 512]) -> Entry {
        let header = Header::from_block(block);

        if header.is_null() {
            self.done = true;
        } else {
            for _ in 0..num_data_blocks(header.size()) {
                self.iter.next();
            }
        }

        Entry { header }
    }
}

impl<T: Read> Iterator for EntryIterator<T> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        if self.done {
            return None;
        }

        self.iter.next().map(|b| self._make_entry(b))
    }
}

#[test]
fn test_entry_iterator() {
    let file = File::open("fixtures/simple.tar").unwrap();
    let subject = EntryIterator::from_stream(file);
    let entries: Vec<Entry> = subject.collect();
    assert_eq!(3, entries.len());
    assert_eq!("1", entries[0].header.name().unwrap());
    assert_eq!("513", entries[1].header.name().unwrap());
    assert_eq!(true, entries[2].header.is_null());
}
