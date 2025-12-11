use bincode::{Decode, Encode};

pub const fn bytes_for_bits(n: usize) -> usize {
    n.div_ceil(8)
}

#[derive(Encode, Decode, Copy, Clone)]
pub struct BitArray<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> BitArray<N> {
    pub fn bit_count(&self) -> usize {
        self.data.len() * 8
    }

    pub fn raw(&self) -> &[u8; N] {
        &self.data
    }

    pub fn raw_mut(&mut self) -> &mut [u8; N] {
        &mut self.data
    }

    pub fn set_bit<T: Into<usize>>(&mut self, n: T, value: bool) {
        let n = n.into();
        let byte_index = n / 8;
        let bit_index = n % 8;
        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }

    pub fn get_bit(&self, n: usize) -> bool {
        let byte_index = n / 8;
        let bit_index = n % 8;
        (self.data[byte_index] & (1 << bit_index)) != 0
    }

    pub fn get_raw(&self) -> &[u8] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|i| *i == 0)
    }

    pub fn random_set_bit(&self, rng: &mut fastrand::Rng) -> Option<usize> {
        let mut total_set_bits = 0;
        for byte in &self.data {
            total_set_bits += byte.count_ones() as usize;
        }

        if total_set_bits == 0 {
            return None;
        }

        let mut target = rng.usize(0..total_set_bits);

        for (byte_index, byte) in self.data.iter().enumerate() {
            for bit_index in 0..8 {
                if byte & (1 << bit_index) != 0 {
                    if target == 0 {
                        return Some(byte_index * 8 + bit_index);
                    }
                    target -= 1;
                }
            }
        }

        None
    }

    pub fn next_set_bit(&self, mut from: usize) -> Option<usize> {
        let len = self.bit_count();
        while from < len {
            if self.get_bit(from) {
                return Some(from);
            }
            from += 1;
        }
        None
    }

    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|i| *i = 0);
    }
}

impl<const N: usize> Default for BitArray<N> {
    fn default() -> Self {
        Self { data: [0; N] }
    }
}

pub struct BitArrayIter<'a, const N: usize> {
    bit_array: &'a BitArray<N>,
    pos: usize,
    total_bits: usize,
}

impl<'a, const N: usize> IntoIterator for &'a BitArray<N> {
    type Item = usize;
    type IntoIter = BitArrayIter<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        BitArrayIter {
            bit_array: self,
            pos: 0,
            total_bits: N * 8,
        }
    }
}

impl<const N: usize> Iterator for BitArrayIter<'_, N> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.total_bits {
            return None;
        }
        let index = self.pos;
        let value = self.bit_array.get_bit(index);
        self.pos += 1;
        if !value {
            return self.next();
        }
        Some(index)
    }
}
