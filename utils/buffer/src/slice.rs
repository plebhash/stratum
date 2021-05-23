use alloc::{sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU8, Ordering};
#[cfg(feature = "debug")]
use std::time::SystemTime;

pub const INGORE_INDEX: u8 = 59;

unsafe impl Send for Slice {}

#[derive(Debug)]
pub struct Slice {
    pub(crate) offset: *mut u8,
    pub(crate) len: usize,
    pub index: u8,
    pub shared_state: SharedState,
    pub owned: Option<Vec<u8>>,
    #[cfg(feature = "debug")]
    pub mode: u8,
    #[cfg(feature = "debug")]
    pub time: SystemTime,
}

impl AsMut<[u8]> for Slice {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        match self.owned.as_mut() {
            None => unsafe { core::slice::from_raw_parts_mut(self.offset, self.len) },
            Some(x) => x,
            //Some(x) => &mut x[0..0],
        }
    }
}

impl Drop for Slice {
    fn drop(&mut self) {
        #[cfg(feature = "debug")]
        self.shared_state.toogle(self.index, self.mode);
        #[cfg(not(feature = "debug"))]
        self.shared_state.toogle(self.index);
    }
}

impl From<Vec<u8>> for Slice {
    fn from(mut v: Vec<u8>) -> Self {
        let offset = v[0..].as_mut_ptr();
        Slice {
            offset,
            len: 0,
            index: crate::slice::INGORE_INDEX,
            shared_state: SharedState::new(),
            owned: Some(v),
            #[cfg(feature = "debug")]
            mode: 2,
            #[cfg(feature = "debug")]
            time: SystemTime::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SharedState(Arc<AtomicU8>);

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self(Arc::new(AtomicU8::new(0)))
    }

    #[inline(always)]
    pub fn load(&self, ordering: Ordering) -> u8 {
        self.0.load(ordering)
    }

    #[cfg(feature = "debug")]
    pub fn toogle(&self, position: u8, mode: u8) {
        let mask: u8 = match position {
            1 => 0b10000000,
            2 => 0b01000000,
            3 => 0b00100000,
            4 => 0b00010000,
            5 => 0b00001000,
            6 => 0b00000100,
            7 => 0b00000010,
            8 => 0b00000001,
            INGORE_INDEX => return,
            _ => panic!("{}", position),
        };
        //if position == 2 {
        //    let bt = Backtrace::force_capture();
        //    println!("{:#?}", bt);
        //};
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |mut shared_state| {
                let pre = shared_state;
                shared_state ^= mask;
                println!("TOOGLE:: {} {:b} {:b}", mode, pre, shared_state);
                Some(shared_state)
            })
            .unwrap();
    }

    #[cfg(not(feature = "debug"))]
    pub fn toogle(&self, position: u8) {
        let mask: u8 = match position {
            1 => 0b10000000,
            2 => 0b01000000,
            3 => 0b00100000,
            4 => 0b00010000,
            5 => 0b00001000,
            6 => 0b00000100,
            7 => 0b00000010,
            8 => 0b00000001,
            INGORE_INDEX => return,
            _ => panic!("{}", position),
        };
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |mut shared_state| {
                shared_state ^= mask;
                Some(shared_state)
            })
            .unwrap();
    }
}
