pub mod data;

pub mod lock {
    use std::{
        ops::{Deref, DerefMut},
        sync::atomic::AtomicBool,
    };

    pub struct CowLock<T>
    where
        T: Clone,
    {
        lock: parking_lot::RwLock<T>,
        join_cache: parking_lot::Mutex<Option<T>>,
        joined: AtomicBool,
    }

    impl<T: Clone> CowLock<T> {
        pub fn new(value: T) -> Self {
            Self {
                lock: parking_lot::RwLock::new(value),
                join_cache: parking_lot::Mutex::new(None),
                joined: AtomicBool::new(true),
            }
        }

        pub fn read<'a>(&'a self) -> parking_lot::RwLockReadGuard<'a, T> {
            self.try_join();
            self.lock.read()
        }

        pub fn write<'a>(&'a self) -> CowLockWriteGuard<'a, T> {
            self.join();

            CowLockWriteGuard {
                lock: self,
                value: Some(self.lock.read().clone()),
            }
        }

        fn try_join(&self) -> bool {
            if self.joined.load(std::sync::atomic::Ordering::Acquire) {
                return true;
            }

            if let Some(mut cache) = self.join_cache.try_lock() {
                if let Some(value) = cache.take() {
                    let mut write = self.lock.write();
                    *write.deref_mut() = value;
                }

                true
            } else {
                false
            }
        }

        fn join(&self) {
            if self.joined.load(std::sync::atomic::Ordering::Acquire) {
                return;
            }

            let mut cache = self.join_cache.lock();
            if let Some(value) = cache.take() {
                let mut write = self.lock.write();
                *write.deref_mut() = value;
            }
        }
    }

    impl<T: Clone> From<T> for CowLock<T> {
        fn from(value: T) -> Self {
            Self::new(value)
        }
    }

    pub struct CowLockWriteGuard<'a, T>
    where
        T: Clone,
    {
        lock: &'a CowLock<T>,
        value: Option<T>,
    }

    impl<T: Clone> Deref for CowLockWriteGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.value.as_ref().unwrap()
        }
    }

    impl<T: Clone> DerefMut for CowLockWriteGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.value.as_mut().unwrap()
        }
    }

    impl<T: Clone> Drop for CowLockWriteGuard<'_, T> {
        fn drop(&mut self) {
            *self.lock.join_cache.lock().deref_mut() = Some(self.value.take().unwrap());

            self.lock
                .joined
                .store(false, std::sync::atomic::Ordering::Release);
        }
    }

    impl<T: Clone + crate::data::Data> crate::data::Data for CowLock<T> {
        fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
            self.lock.read().encode(writer)
        }

        fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
            Ok(Self::new(T::decode(reader)?))
        }

        fn bytes_len(&self) -> usize {
            self.lock.read().bytes_len()
        }
    }
}
