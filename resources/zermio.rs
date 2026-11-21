// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

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
    + BitXor<Output = Self>
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
    reg: &'a Register<T>,
    _data: core::marker::PhantomData<ACCESS>,
}

// Impl block for any access permission.
impl<'a, const OFFSET: usize, const BITS: usize, T, ACCESS> BitField<'a, OFFSET, BITS, T, ACCESS>
where
    T: UnsignedInteger,
{
    pub fn new(reg: &'a Register<T>) -> Self {
        const {
            core::assert!(
                BITS <= core::mem::size_of::<T>() * 8,
                "Bitfield width exceeds type size"
            );
        }
        const {
            core::assert!(
                (BITS + OFFSET) <= core::mem::size_of::<T>() * 8,
                "Offset exceeds type size"
            );
        }
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
        self.max() >= val
    }
}

// Impl block for Write access permission.
impl<const OFFSET: usize, const BITS: usize, T, ACCESS> BitField<'_, OFFSET, BITS, T, ACCESS>
where
    ACCESS: access::Writable,
    T: UnsignedInteger,
{
    pub fn write(&self, value: T) -> &Self {
        self.clear();
        self.reg
            .cache
            .set(self.reg.cache.get() | (value << T::from(OFFSET)) & self.mask());
        self
    }

    pub fn clear(&self) -> &Self {
        self.reg.cache.set(self.reg.cache.get() & !self.mask());
        self
    }

    pub fn write_mask(&self, val: T) -> &Self {
        let mut cache = self.reg.cache.get() & !(val << T::from(OFFSET) & self.mask());
        cache |= (val << T::from(OFFSET)) & self.mask();
        self.reg.cache.set(cache);
        self
    }

    pub fn commit(&self) {
        self.reg.commit()
    }
}

// Impl block for single bit fields and Write access permission.
impl<const OFFSET: usize, T, ACCESS> BitField<'_, OFFSET, 1, T, ACCESS>
where
    ACCESS: access::Writable,
    T: UnsignedInteger,
{
    pub fn set(&self) -> &Self {
        self.reg
            .cache
            .set(self.reg.cache.get() | T::from(0x01usize << OFFSET));
        self
    }

    pub fn reset(&self) -> &Self {
        self.clear();
        self
    }

    pub fn toggle(&self) -> &Self {
        let cache = self.reg.cache.get() ^ T::from(0x01usize << OFFSET);
        self.reg.cache.set(cache);
        self
    }

    pub fn assign(&self, bit: bool) -> &Self {
        let mut cache = self.reg.cache.get() & !T::from(0x01 << OFFSET);
        cache |= T::from((bit as usize) << OFFSET);
        self.reg.cache.set(cache);
        self
    }
}

// Impl block for Read access permission.
impl<const OFFSET: usize, const BITS: usize, T, ACCESS> BitField<'_, OFFSET, BITS, T, ACCESS>
where
    ACCESS: access::Readable,
    T: UnsignedInteger,
{
    pub fn get(&self) -> T {
        (self.reg.cache.get() & self.mask()) >> T::from(OFFSET)
    }

    pub fn fetch(&self) -> &Self {
        self.reg.fetch();
        self
    }
}

// Impl block for single bit fields and Read access permission.
impl<const OFFSET: usize, T, ACCESS> BitField<'_, OFFSET, 1, T, ACCESS>
where
    ACCESS: access::Readable,
    T: UnsignedInteger,
{
    pub fn is_set(&self) -> bool {
        (self.reg.cache.get() & self.mask()) == self.mask()
    }
}

pub struct Register<T> {
    pub cache: core::cell::Cell<T>,
    ptr: *mut T,
}

unsafe impl<T> Send for Register<T> {}
unsafe impl<T> Sync for Register<T> {}

impl<T> Register<T>
where
    T: UnsignedInteger,
{
    pub fn new(addr: usize) -> Self {
        Self {
            ptr: addr as *mut T,
            cache: core::cell::Cell::new(T::from(0usize)),
        }
    }

    pub fn commit(&self) {
        unsafe { self.ptr.write_volatile(self.cache.get()) };
    }

    pub fn fetch(&self) -> &Self {
        unsafe { self.cache.set(self.ptr.read_volatile()) };
        self
    }
}

pub struct Memory<T> {
    ptr: *mut T,
    size: usize,
}

impl<T> Memory<T>
where
    T: UnsignedInteger,
{
    pub fn new(addr: usize, size: usize) -> Self {
        Self {
            ptr: addr as *mut T,
            size,
        }
    }

    /// Writes data to a FIFO memory window at a fixed destination address.
    ///
    /// Unlike a standard buffer write, this function performs multiple writes to the
    /// **same physical address**. Each write transaction pushes data into the hardware FIFO.
    ///
    /// # Important
    /// - **Alignment**: It automatically handles unaligned source pointers by performing
    ///   byte-level writes until `T`-alignment is reached.
    /// - **Transaction Size**: Uses the native type `T` for the bulk of the transfer to
    ///   ensure the hardware receives correctly sized bus transactions.
    /// - **Usage**: Do NOT use this for standard RAM buffers; it does not increment
    ///   the destination address.
    pub fn write_fifo(&self, src: &[u8]) {
        let t_size = core::mem::size_of::<T>();
        // 1. Peel: Byte-wise volatile writes for alignment
        let align_offset = src.as_ptr().align_offset(t_size);
        let len = core::cmp::min(src.len(), align_offset);
        if len > 0 {
            for byte in src.iter().take(len) {
                unsafe {
                    core::ptr::write_volatile(self.ptr as *mut u8, *byte);
                }
            }
        }

        // 2. Loop: Native T volatile writes
        let chunk_size = t_size;
        let mut iter = src[len..].chunks_exact(chunk_size);

        for chunk in &mut iter {
            unsafe {
                // We cast to the native type T to trigger the correct bus width (e.g., 32-bit)
                let value = *(chunk.as_ptr() as *const T);
                core::ptr::write_volatile(self.ptr, value);
            }
        }

        // 3. Remainder: Trailing bytes
        let remainder = iter.remainder();
        for &byte in remainder {
            unsafe {
                core::ptr::write_volatile(self.ptr as *mut u8, byte);
            }
        }
    }

    /// Writes data to a memory buffer.
    ///
    /// # Important
    /// - **Alignment**: The src slice must by aligned by size_of<T>.
    pub fn write_buffer(&self, src: &[u8]) -> usize {
        let t_size = core::mem::size_of::<T>();
        let align_offset = src.as_ptr().align_offset(t_size);

        let chunk_size = if align_offset == 0 {
            t_size
        } else {
            // If it's not aligned, skip the next for loop and copy all as bytes in the reminder
            // loop.
            src.len() + 1
        };
        let mut iter = src.chunks_exact(chunk_size);

        let mut dst = self.ptr;
        let upper_bound = unsafe { dst.add(self.size) };
        for chunk in &mut iter {
            unsafe {
                let value = *(chunk.as_ptr() as *const T);
                core::ptr::write_volatile(dst, value);
                dst = dst.add(1);
            }
            if dst >= upper_bound {
                return self.size * t_size;
            }
        }

        let remainder = iter.remainder();
        let mut dst = dst as *mut u8;
        let upper_bound = upper_bound as *mut u8;
        for &byte in remainder {
            unsafe {
                core::ptr::write_volatile(dst, byte);
                dst = dst.add(1);
            }
            if dst >= upper_bound {
                return self.size * t_size;
            }
        }
        src.len()
    }

    pub fn read(&self, dst: &mut [u8]) {
        unsafe { core::ptr::copy_nonoverlapping(self.ptr as *mut u8, dst.as_mut_ptr(), dst.len()) };
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod unittest {

    #[test]
    fn test_memory_window() {
        let mut arr = [0u32; 8];
        let window = Memory::<u32>::new(arr.as_ptr() as usize, 8);
        let test_data = [0xbe, 0xba, 0xca, 0xfe];
        window.write_buffer(&test_data);
        assert_eq!(arr[0], u32::from_ne_bytes(test_data));

        let mut read_back = [0xff; 4];
        window.read(&mut read_back);
        assert_eq!(read_back, test_data);

        arr[0] = 0x00;
        let test_data = [0xbe, 0xba];
        window.write_buffer(&test_data);
        assert_eq!(arr[0], u32::from_ne_bytes([0xbe, 0xba, 0x00, 00]));

        arr[0] = 0x00;
        let test_data = [0x55, 0x44, 0x83, 0xa3, 0xb4, 0x55];
        window.write_buffer(&test_data);
        assert_eq!(arr[0], u32::from_ne_bytes([0x55, 0x44, 0x83, 0xa3]));
        assert_eq!(arr[1], u32::from_ne_bytes([0xb4, 0x55, 00, 00]));
    }

    #[test]
    fn test_memory_window_single_byte_unaligned() {
        let arr = std::cell::UnsafeCell::new([0u32; 2]);
        let window = Memory::<u32>::new(arr.get() as usize, 8);
        let test_data = [0xbe, 0xba, 0xca, 0xfe];
        window.write_buffer(&test_data[2..3]);
        assert_eq!(
            unsafe { (*arr.get())[0] },
            u32::from_ne_bytes([0xca, 0x00, 0x00, 0x00])
        );
    }

    #[test]
    fn test_memory_window_unaligned() {
        let arr = std::cell::UnsafeCell::new([0u32; 2]);
        let window = Memory::<u32>::new(arr.get() as usize, 8);
        let test_data = [0xbe, 0xba, 0xca, 0xfe];
        window.write_buffer(&test_data[1..]);
        assert_eq!(
            unsafe { (*arr.get())[0] },
            u32::from_ne_bytes([0xba, 0xca, 0xfe, 0x00])
        );
    }

    #[test]
    fn test_memory_fifo_alignment() {
        let arr = std::cell::UnsafeCell::new([0u32; 2]);
        let window = Memory::<u32>::new(arr.get() as usize, 8);
        let test_data = [0x55, 0x44, 0x83, 0xa3, 0xb4];
        window.write_fifo(&test_data[1..]);
        assert_eq!(
            unsafe { (*arr.get())[0] },
            u32::from_ne_bytes([0x44, 0x83, 0xa3, 0xb4])
        );
    }

    #[test]
    fn test_memory_fifo_single_byte_aligned() {
        let arr = std::cell::UnsafeCell::new([0u32; 2]);
        let window = Memory::<u32>::new(arr.get() as usize, 8);
        let test_data = [0x55, 0x44, 0x83, 0xa3, 0xb4];
        window.write_fifo(&test_data[2..3]);
        assert_eq!(
            unsafe { (*arr.get())[0] },
            u32::from_ne_bytes([0x83, 0x00, 0x00, 0x00])
        );
    }

    #[test]
    fn test_memory_fifo_alignment_multiword() {
        let arr = std::cell::UnsafeCell::new([0u32; 2]);
        let window = Memory::<u32>::new(arr.get() as usize, 8);
        let test_data = [0x55, 0x44, 0x83, 0xa3, 0xb4, 0x55, 0x23, 0x34];
        window.write_fifo(&test_data[1..]);
        assert_eq!(
            unsafe { (*arr.get())[0] },
            u32::from_ne_bytes([0xb4, 0x55, 0x23, 0x34])
        );
    }

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
        let reg = Register::<u64>::new((&mem as *const u64) as usize);
        assert_eq!(
            BitField::<0, 64, u64, access::ReadWrite>::new(&reg).max(),
            u64::MAX
        );
    }

    #[test]
    fn test_mask() {
        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);

        assert_eq!(
            BitField::<10, 1, u32, access::ReadWrite>::new(&reg).mask(),
            1 << 10
        );
        assert_eq!(
            BitField::<10, 2, u32, access::ReadWrite>::new(&reg).mask(),
            3 << 10
        );
        assert_eq!(
            BitField::<0, 10, u32, access::ReadWrite>::new(&reg).mask(),
            (1 << 10) - 1
        );
        assert_eq!(
            BitField::<1, 10, u32, access::ReadWrite>::new(&reg).mask(),
            ((1 << 10) - 1) << 1
        );
        assert_eq!(
            BitField::<0, 32, u32, access::ReadWrite>::new(&reg).mask(),
            u32::MAX
        );

        let mem = 0u64;
        let reg = Register::<u64>::new((&mem as *const u64) as usize);
        assert_eq!(
            BitField::<0, 64, u64, access::ReadWrite>::new(&reg).mask(),
            u64::MAX
        );
    }

    #[test]
    fn test_set_reset() {
        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<10, 1, u32, access::ReadWrite>::new(&reg);
        field.set().commit();
        assert_eq!(mem, 1 << 10);
        field.reset().commit();
        assert_eq!(mem, 0);
    }

    #[test]
    fn test_fetch_reset() {
        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        reg.fetch();
        let field = BitField::<15, 1, u32, access::ReadWrite>::new(&reg);
        field.reset().commit();
        assert_eq!(mem, !(1 << 15));
    }

    #[test]
    fn test_toggle() {
        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<30, 1, u32, access::ReadWrite>::new(&reg);
        field.toggle().commit();
        assert_eq!(mem, 1 << 30);
        field.toggle().commit();
        assert_eq!(mem, 0);
        field.toggle().commit();
        assert_eq!(mem, 1 << 30);

        let field = BitField::<10, 1, u32, access::ReadWrite>::new(&reg);
        field.toggle().commit();
        assert_eq!(mem, 1 << 30 | 1 << 10);
    }

    #[test]
    fn test_is_set() {
        let mem = 1u32 << 0;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<0, 1, u32, access::ReadWrite>::new(&reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 1;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<1, 1, u32, access::ReadWrite>::new(&reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 15;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<15, 1, u32, access::ReadWrite>::new(&reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 31;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<31, 1, u32, access::ReadWrite>::new(&reg);
        assert_eq!(field.fetch().is_set(), true);

        let mem = 1u32 << 30;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<30, 1, u32, access::ReadWrite>::new(&reg);
        assert_eq!(field.fetch().is_set(), true);
    }

    #[test]
    fn test_reset() {
        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<31, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 31) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<30, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 30) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<15, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 15) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<1, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 1) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<0, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().reset().commit();
        assert_eq!(mem, !(1 << 0) & u32::MAX);
    }

    #[test]
    fn test_set() {
        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<31, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 31));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<30, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 30));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<15, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 15));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<1, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 1));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<0, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().set().commit();
        assert_eq!(mem, (1 << 0));
    }

    #[test]
    fn test_assign_true() {
        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<31, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 31));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<30, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 30));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<15, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 15));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<1, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 1));

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<0, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(true).commit();
        assert_eq!(mem, (1 << 0));
    }

    #[test]
    fn test_assign_false() {
        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<31, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 31) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<30, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 30) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<15, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 15) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<1, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 1) & u32::MAX);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<0, 1, u32, access::ReadWrite>::new(&reg);
        field.fetch().assign(false).commit();
        assert_eq!(mem, !(1 << 0) & u32::MAX);
    }

    #[test]
    fn test_clear() {
        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        reg.fetch();
        let field = BitField::<5, 8, u32, access::ReadWrite>::new(&reg);
        field.clear().commit();
        assert_eq!(mem, u32::MAX & !(0xff << 5));
    }

    #[test]
    fn test_write() {
        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<5, 8, u32, access::ReadWrite>::new(&reg);

        field.write(0xfa).commit();
        assert_eq!(mem, 0xfa << 5);

        field.write(0x7a).commit();
        assert_eq!(mem, 0x7a << 5);

        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<5, 8, u32, access::ReadWrite>::new(&reg);
        field.fetch().write(0x7a).commit();
        assert_eq!(mem, (u32::MAX & !(0xff << 5)) | (0x7a << 5));
    }

    #[test]
    fn test_write_mask() {
        let mem = u32::MAX;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<5, 8, u32, access::ReadWrite>::new(&reg);
        field.write_mask(0xfa).commit();
        assert_eq!(mem, 0xfa << 5);

        let mem = 0u32;
        let reg = Register::<u32>::new((&mem as *const u32) as usize);
        let field = BitField::<5, 8, u32, access::ReadWrite>::new(&reg);
        field.fetch().write(0x80).commit();
        field.fetch().write_mask(0x1 << 2).commit();
        assert_eq!(mem, (0x80 | (0x01 << 2)) << 5);
    }
}
