use std::{io, ops::{Index, IndexMut, Range, RangeInclusive}};

use memmap2::{MmapMut, MmapOptions};

pub struct MonitoredMemory {
    inner: MmapMut,
}

impl MonitoredMemory {
    /// Creates a new instance of [`MonitoredMemory`] with the specified `size`.
    pub fn new(size: usize) -> io::Result<Self> {
        let inner = MmapOptions::new().len(size).map_anon()?;
        Ok(MonitoredMemory { inner })
    }

    /// Returns the size of the [`MonitoredMemory`].
    pub fn size(&self) -> usize {
        self.inner.len()
    }
}

impl Index<usize> for MonitoredMemory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for MonitoredMemory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl Index<Range<usize>> for MonitoredMemory {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<Range<usize>> for MonitoredMemory {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl Index<RangeInclusive<usize>> for MonitoredMemory {
    type Output = [u8];

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<RangeInclusive<usize>> for MonitoredMemory {
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        &mut self.inner[index]
    }
}



