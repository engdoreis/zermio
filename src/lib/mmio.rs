pub struct Peripheral<'a> {
    pub name: &'a str,
    pub registers: Vec<&'a str>,
}

impl<'a> Peripheral<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            registers: vec![],
        }
    }
}

pub struct Register<'a> {
    pub name: &'a str,
    pub offset: String,
    pub bitfields: Vec<Bitfields<'a>>,
}

impl<'a> Register<'a> {
    pub fn new(name: &'a str, offset: u32) -> Self {
        Self {
            name,
            offset: format!("{:#x}", offset),
            bitfields: vec![],
        }
    }
}

pub struct Bitfields<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub bit_size: u32,
    pub offset: u32,
}

impl<'a> Bitfields<'a> {
    pub fn new(name: &'a str, offset: u32, bit_size: u32) -> Self {
        Self {
            name,
            offset,
            bit_size,
            desc: name,
        }
    }
}
