use std::io::{self, Read, Stdin};
use std::string::FromUtf8Error;

fn main() {
    let mut stdin = io::stdin();
    let block = read_block(&mut stdin);
    let header = Header::from_block(block);

    if header.has_magic() {
        println!("This looks like a valid tar file.");

        match header.name() {
            Ok(n) => println!("The name of the first entry is {}", n),
            Err(e) => println!("The first entry's name is invalid: {}", e)
        };
    } else {
        println!("Bad magic in the first header.");
    }
}

fn read_block(stdin: &mut Stdin) -> [u8; 512] {
    let mut block: [u8; 512] = [0; 512];
    match stdin.read(&mut block) {
        Ok(512) => block,
        Ok(n) => panic!("Expected to read 512 bytes but got {}", n),
        Err(e) => panic!("Read error: {}", e)
    }
}

struct Header {
    block: [u8; 512]
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
    fn from_block(block: [u8;512]) -> Header {
        Header { block }
    }

    fn has_magic(self: &Header) -> bool {
        let maybe_magic = String::from_utf8(self.magic_field().to_vec());

        match maybe_magic {
            Ok(actual) => actual == String::from("ustar\0"),
            _ => false,
        }
    }

    fn name(self: &Header) -> Result<String, FromUtf8Error> {
        let len = find_zero(&self.block, NAME_LEN).unwrap_or(NAME_LEN);
        let bytes = self.block[0..len].to_vec();
        String::from_utf8(bytes)
    }

    fn magic_field(self: &Header) -> &[u8] {
        let offset = NAME_LEN + MODE_LEN + UID_LEN + GID_LEN + SIZE_LEN +
            MTIME_LEN + CHECKSUM_LEN + TYPEFLAG_LEN + LINKNAME_LEN;
        &self.block[offset..(offset + MAGIC_LEN)]
    }
}

fn find_zero(buf: &[u8; 512], maxlen: usize) -> Option<usize> {
    for i in 0..maxlen {
        if buf[i] == 0 {
            return Some(i);
        }
    }

    None
}


#[test]
fn test_has_magic() {
    let good = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000000 13124523641 013414^@ 0^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@ustar");
    let bad = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@000644 ^@000765 ^@000024 ^@00000000000 13124523641 013414^@ 0^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@nope!");
    assert_eq!(true, Header::from_block(good).has_magic());
    assert_eq!(false, Header::from_block(bad).has_magic());
}

#[test]
fn test_name_short() {
    let block = block_from_visual("somefile^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@x");
    assert_eq!("somefile", Header::from_block(block).name().unwrap());
}

#[test]
fn test_name_exactly_100() {
    let block = block_from_visual("long________________________________________________________________________________________________x");
    assert_eq!("long________________________________________________________________________________________________", Header::from_block(block).name().unwrap());
}

fn block_from_visual(visual: &str) -> [u8;512] {
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
