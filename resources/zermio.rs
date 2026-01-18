use core::cmp::PartialEq;
use core::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub,
};

// Define access control for the registers.
pub mod access {
    pub trait Readable {}
    pub trait Writable {}

    pub struct Read {}
    impl Readable for Read {}

    pub struct ReadWrite {}
    impl Readable for ReadWrite {}
    impl Writable for ReadWrite {}

    pub struct Write {}
    impl Writable for Write {}
}

// Super trait to constraint an integer type.
pub trait UnsignedInteger:
    Add<Output = Self>
    + Sub<Output = Self>
    + Shl<Output = Self>
    + Shr<Output = Self>
    + BitAnd<Output = Self>
    + Not<Output = Self>
    + BitOr<Output = Self>
    + BitOrAssign<Self>
    + BitAndAssign<Self>
    + BitXor<Self>
    + BitXorAssign<Self>
    + PartialEq
    + PartialOrd
    + Copy
{
    fn from(x: usize) -> Self;
    fn max() -> Self;
}

impl UnsignedInteger for u32 {
    fn from(x: usize) -> Self {
        x as u32
    }
    fn max() -> Self {
        u32::MAX
    }
}

impl UnsignedInteger for u64 {
    fn from(x: usize) -> Self {
        x as u64
    }
    fn max() -> Self {
        u64::MAX
    }
}

// Define the BitField type with ReadWrite access as default
pub struct BitField<'a, const OFFSET: usize, const BITS: usize, T, ACCESS = access::ReadWrite> {
    reg: &'a mut Register<T>,
    _data: core::marker::PhantomData<ACCESS>,
}

// Impl block for any access permission.
impl<'a, const OFFSET: usize, const BITS: usize, T, ACCESS> BitField<'a, OFFSET, BITS, T, ACCESS>
where
    T: UnsignedInteger,
{
    const _CHECK: () = assert!(
        BITS <= core::mem::size_of::<T>() * 8,
        "Bitfield width exceeds type size"
    );
    const _CHECK2: () = assert!(
        (BITS + OFFSET) > core::mem::size_of::<T>() * 8,
        "Offset exceeds type size"
    );

    pub fn new(reg: &'a mut Register<T>) -> Self {
        Self {
            reg,
            _data: core::marker::PhantomData::<ACCESS>,
        }
    }

    pub fn mask(&self) -> T {
        if BITS == size_of::<T>() * 8usize {
            return T::max();
        }
        T::from(((0x01 << BITS) - 1) << OFFSET)
    }

    pub fn max(&self) -> T {
        if BITS == size_of::<T>() * 8usize {
            return T::max();
        }
        T::from((1 << BITS) - 1)
    }

    pub fn in_range(&self, val: T) -> bool {
        self.max() <= val
    }
}

// Impl block for Write access permission.
impl<'a, const OFFSET: usize, const BITS: usize, T, ACCESS> BitField<'a, OFFSET, BITS, T, ACCESS>
where
    ACCESS: access::Writable,
    T: UnsignedInteger,
{
    pub fn write(&mut self, value: T) -> &mut Self {
        self.clear();
        self.reg.cache |= (value << T::from(OFFSET)) & self.mask();
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.reg.cache &= !self.mask();
        self
    }

    pub fn write_mask(&mut self, val: T) -> &mut Self {
        self.reg.cache &= !(val << T::from(OFFSET) & self.mask());
        self.reg.cache |= (val << T::from(OFFSET)) & self.mask();
        self
    }

    pub fn commit(&mut self) {
        self.reg.commit()
    }
}

// Impl block for single bit fields and Write access permission.
impl<'a, const OFFSET: usize, T, ACCESS> BitField<'a, OFFSET, 1, T, ACCESS>
where
    ACCESS: access::Writable,
    T: UnsignedInteger,
{
    pub fn set(&mut self) -> &mut Self {
        self.reg.cache |= T::from(0x01usize << OFFSET);
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.clear();
        self
    }

    pub fn toggle(&mut self) -> &mut Self {
        self.reg.cache ^= T::from(0x01usize << OFFSET);
        self
    }

    pub fn assign(&mut self, bit: bool) -> &mut Self {
        self.reg.cache &= !T::from(0x01 << OFFSET);
        self.reg.cache |= T::from((bit as usize) << OFFSET);
        self
    }
}

// Impl block for Read access permission.
impl<'a, const OFFSET: usize, const BITS: usize, T, ACCESS> BitField<'a, OFFSET, BITS, T, ACCESS>
where
    ACCESS: access::Readable,
    T: UnsignedInteger,
{
    pub fn get(&self) -> T {
        (self.reg.cache & self.mask()) >> T::from(OFFSET)
    }

    pub fn fetch(&mut self) -> &mut Self {
        self.reg.fetch();
        self
    }
}

// Impl block for single bit fields and Read access permission.
impl<'a, const OFFSET: usize, T, ACCESS> BitField<'a, OFFSET, 1, T, ACCESS>
where
    ACCESS: access::Readable,
    T: UnsignedInteger,
{
    pub fn is_set(&self) -> bool {
        (self.reg.cache & self.mask()) == self.mask()
    }
}

pub struct Register<T> {
    pub cache: T,
    ptr: *mut T,
}

impl<T> Register<T>
where
    T: UnsignedInteger,
{
    pub fn new(addr: usize) -> Self {
        Self {
            ptr: addr as *mut T,
            cache: T::from(0usize),
        }
    }

    pub fn commit(&mut self) {
        unsafe { self.ptr.write_volatile(self.cache) };
    }

    pub fn fetch(&mut self) -> &mut Self {
        unsafe { self.cache = self.ptr.read_volatile() };
        self
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod unittest {
    use super::*;
    #[test]
    fn test_max() {
        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);

        assert_eq!(
            BitField::<10, 1, u32, access::ReadWrite>::new(&mut reg).max(),
            1
        );
        assert_eq!(
            BitField::<10, 2, u32, access::ReadWrite>::new(&mut reg).max(),
            (1 << 2) - 1
        );
        assert_eq!(
            BitField::<0, 10, u32, access::ReadWrite>::new(&mut reg).max(),
            (1 << 10) - 1
        );
        assert_eq!(
            BitField::<1, 10, u32, access::ReadWrite>::new(&mut reg).max(),
            (1 << 10) - 1
        );
        assert_eq!(
            BitField::<0, 32, u32, access::ReadWrite>::new(&mut reg).max(),
            u32::MAX
        );

        let mem = 0u64;
        let mut reg = Register::<u64>::new((&mem as *const u64) as usize);
        assert_eq!(
            BitField::<0, 64, u64, access::ReadWrite>::new(&mut reg).max(),
            u64::MAX
        );
        assert_eq!(
            BitField::<1, 64, u64, access::ReadWrite>::new(&mut reg).max(),
            u64::MAX
        );
    }

    #[test]
    fn test_mask() {
        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);

        assert_eq!(
            BitField::<10, 1, u32, access::ReadWrite>::new(&mut reg).mask(),
            1 << 10
        );
        assert_eq!(
            BitField::<10, 2, u32, access::ReadWrite>::new(&mut reg).mask(),
            3 << 10
        );
        assert_eq!(
            BitField::<0, 10, u32, access::ReadWrite>::new(&mut reg).mask(),
            (1 << 10) - 1
        );
        assert_eq!(
            BitField::<1, 10, u32, access::ReadWrite>::new(&mut reg).mask(),
            ((1 << 10) - 1) << 1
        );
        assert_eq!(
            BitField::<0, 32, u32, access::ReadWrite>::new(&mut reg).mask(),
            u32::MAX
        );

        let mem = 0u64;
        let mut reg = Register::<u64>::new((&mem as *const u64) as usize);
        assert_eq!(
            BitField::<0, 64, u64, access::ReadWrite>::new(&mut reg).mask(),
            u64::MAX
        );
    }

    #[test]
    fn test_set_reset() {
        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<10, 1, u32, access::ReadWrite>::new(&mut reg);
        field.set().commit();
        assert_eq!(mem, 1 << 10);
        field.reset().commit();
        assert_eq!(mem, 0);
    }

    #[test]
    fn test_fetch_reset() {
        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        reg.fetch();
        let mut field = BitField::<15, 1, u32, access::ReadWrite>::new(&mut reg);
        field.reset().commit();
        assert_eq!(mem, !(1 << 15));
    }

    #[test]
    fn test_toggle() {
        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<30, 1, u32, access::ReadWrite>::new(&mut reg);
        field.toggle().commit();
        assert_eq!(mem, 1 << 30);
        field.toggle().commit();
        assert_eq!(mem, 0);
        field.toggle().commit();
        assert_eq!(mem, 1 << 30);

        let mut field = BitField::<10, 1, u32, access::ReadWrite>::new(&mut reg);
        field.toggle().commit();
        assert_eq!(mem, 1 << 30 | 1 << 10);
    }

    #[test]
    fn test_is_set() {
        let mem = 1u32 << 0;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<0, 1, u32, access::ReadWrite>::new(&mut reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 1;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<1, 1, u32, access::ReadWrite>::new(&mut reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 15;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<15, 1, u32, access::ReadWrite>::new(&mut reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 31;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<31, 1, u32, access::ReadWrite>::new(&mut reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 30;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<30, 1, u32, access::ReadWrite>::new(&mut reg);
        assert_eq!(field.fetch().is_set(), true);
    }

    #[test]
    fn test_reset() {
        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<31, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 31) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<30, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 30) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<15, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 15) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<1, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 1) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<0, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 0) & u32::MAX);
    }

    #[test]
    fn test_set() {
        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<31, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 31));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<30, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 30));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<15, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 15));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<1, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 1));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<0, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 0));
    }

    #[test]
    fn test_assign_true() {
        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<31, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 31));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<30, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 30));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<15, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 15));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<1, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 1));

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<0, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 0));
    }

    #[test]
    fn test_assign_false() {
        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<31, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 31) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<30, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 30) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<15, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 15) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<1, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 1) & u32::MAX);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<0, 1, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 0) & u32::MAX);
    }

    #[test]
    fn test_clear() {
        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        reg.fetch();
        let mut field = BitField::<5, 8, u32, access::ReadWrite>::new(&mut reg);
        field.clear().commit();
        assert_eq!(mem, u32::MAX & !(0xff << 5));
    }

    #[test]
    fn test_write() {
        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<5, 8, u32, access::ReadWrite>::new(&mut reg);

        field.write(0xfa).commit();
        assert_eq!(mem, 0xfa << 5);

        field.write(0x7a).commit();
        assert_eq!(mem, 0x7a << 5);

        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<5, 8, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().write(0x7a).commit();
        assert_eq!(mem, (u32::MAX & !(0xff << 5)) | (0x7a << 5));
    }

    #[test]
    fn test_write_mask() {
        let mem = u32::MAX;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<5, 8, u32, access::ReadWrite>::new(&mut reg);
        field.write_mask(0xfa).commit();
        assert_eq!(mem, 0xfa << 5);

        let mem = 0u32;
        let mut reg = Register::<u32>::new((&mem as *const u32) as usize);
        let mut field = BitField::<5, 8, u32, access::ReadWrite>::new(&mut reg);
        field.fetch().write(0x80).commit();
        field.fetch().write_mask(0x1 << 2).commit();
        assert_eq!(mem, (0x80 | (0x01 << 2)) << 5);
    }
}
