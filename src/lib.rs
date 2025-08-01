pub mod files;

pub trait Serialize {
    fn write(&self, out: &mut Vec<u8>);
}
pub trait Deserialize<'a>: Sized {
    fn read(bytes: &'a [u8]) -> (Self, usize);
}

#[derive(Clone, Copy, Debug)]
pub struct Section<'a> {
    pub ty: u32,
    pub metadata: u32,
    pub bytes: &'a [u8]
}
impl<'a> Serialize for Section<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.ty.to_be_bytes());
        out.extend_from_slice(&self.metadata.to_be_bytes());
        out.extend_from_slice(&(self.bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(self.bytes);
    }
}
impl<'a> Deserialize<'a> for Section<'a> {
    fn read(bytes: &'a [u8]) -> (Self, usize) {
        let ty = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let metadata = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let len = u32::from_be_bytes(bytes[8..12].try_into().unwrap());
        let bytes = &bytes[12..(12 + len as usize)];
        (Self {
            ty,
            metadata,
            bytes,
        }, (12 + len as usize))
    }
}

#[derive(Debug, Clone)]
pub struct File<'a> {
    pub magic: [u8; 4],
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub sections: Vec<Section<'a>>
}
impl<'a> Serialize for File<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.magic);
        out.push(0);
        out.push(self.version_major);
        out.push(self.version_minor);
        out.push(self.version_patch);
        for section in &self.sections {
            section.write(out);
        }
    }
}
impl<'a> Deserialize<'a> for File<'a> {
    fn read(bytes: &'a [u8]) -> (Self, usize) {
        let magic = bytes[0..4].try_into().unwrap();
        assert_eq!(magic, [b'n', b't', b'a', b'r']);
        let version_major = bytes[5];
        let version_minor = bytes[6];
        let version_patch = bytes[7];
        let mut ptr = 8;
        let mut sections = vec![];
        while ptr < bytes.len() {
            let (section, incr) = Section::read(&bytes[ptr..]);
            sections.push(section);
            ptr += incr
        }
        (Self {
            magic,
            version_major,
            version_minor,
            version_patch,
            sections,
        }, ptr)
    }
}
impl<'a> File<'a> {
    pub fn new(sections: Vec<Section<'a>>) -> Self {
        Self { magic: [b'n', b't', b'a', b'r'], version_major: 0, version_minor: 0, version_patch: 0, sections }
    }
    pub fn sanity_check(&self) {
        assert_eq!(self.magic, [b'n', b't', b'a', b'r']);
    }
}
