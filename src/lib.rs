pub use paste::paste;

pub trait NonZeroEquivalent {
    type NonZeroEquivalent;
}

pub trait Primitive {
    type Primative;
}

macro_rules! impl_relationship {
    ($unsigned:ty, $non_zero:path) => {
        impl NonZeroEquivalent for $unsigned {
            type NonZeroEquivalent = $non_zero;
        }

        impl Primitive for $non_zero {
            type Primative = $unsigned;
        }
    };
}

impl_relationship!(u8, std::num::NonZeroU8);
impl_relationship!(u16, std::num::NonZeroU16);
impl_relationship!(u32, std::num::NonZeroU32);
impl_relationship!(u64, std::num::NonZeroU64);
impl_relationship!(u128, std::num::NonZeroU128);
impl_relationship!(usize, std::num::NonZeroUsize);

pub trait ReadonlyStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
}

pub trait MutableStorage {
    fn set(&mut self, key: &[u8], value: &[u8]);

    fn clear(&mut self, key: &[u8]);
}

pub mod item {
    use crate::{MutableStorage, ReadonlyStorage};

    pub trait Store: Sized {
        fn load(storage: &dyn ReadonlyStorage) -> Option<Self>;

        fn save(&self, storage: &mut dyn MutableStorage);
    }

    pub trait Clear {
        fn clear(storage: &mut dyn MutableStorage);
    }

    pub trait LoadAlways: Sized {
        fn load_always(storage: &dyn ReadonlyStorage) -> Self;
    }

    /// marker trait, making `Clear` & `LoadAlways` mutually exclusive
    pub trait ClearOrLoadAlways {}

    #[macro_export]
    macro_rules! item_store_derive_attrs {
        ($Item:ident, custom(item_store(always))) => {
            impl $crate::item::ClearOrLoadAlways for $Item {}

            impl $crate::item::LoadAlways for $Item {
                fn load_always(storage: &dyn $crate::ReadonlyStorage) -> Self {
                    Self::load(storage).expect("always present in storage")
                }
            }
        };
        ($Item:ident, custom(item_store(clear))) => {
            impl $crate::item::ClearOrLoadAlways for $Item {}

            impl $crate::item::Clear for $Item {
                fn clear(storage: &mut dyn $crate::MutableStorage) {
                    storage.clear(Self::KEY.as_bytes());
                }
            }
        };
        ($_Item:ident, $($_other_meta:tt)+) => {};
    }

    #[macro_export]
    macro_rules! ItemStoreImpl {
        (
        $(#[$($meta_item:tt)+])*
        $pub:vis struct $Item:ident($Inner:ident);
    ) => {
            impl $Item {
                $crate::paste! {
                    const KEY: &'static str = concat!(module_path!(), "::", stringify!([< $Item:snake _ $Inner:snake >]));
                }
            }

            impl $crate::item::Store for $Item {
                fn load(storage: &dyn $crate::ReadonlyStorage) -> Option<Self> {
                    storage.get(Self::KEY.as_bytes()).map(Self::from_owned_bytes)
                }

                fn save(&self, storage: &mut dyn $crate::MutableStorage) {
                    storage.set(Self::KEY.as_bytes(), self.to_owned_bytes().as_slice());
                }
            }

            $(
                $crate::item_store_derive_attrs!($Item, $($meta_item)+);
            )*
        };
    }
}

pub mod map {
    use crate::{MutableStorage, ReadonlyStorage};

    pub trait IntoMapKey {
        fn into_map_key(self) -> String;
    }

    impl<T1, T2> IntoMapKey for (T1, T2)
    where
        T1: IntoMapKey,
        T2: IntoMapKey,
    {
        fn into_map_key(self) -> String {
            let mut key = self.0.into_map_key();
            key.push(':');
            key.push_str(self.1.into_map_key().as_str());
            key
        }
    }

    macro_rules! impl_to_map_key_uint {
        ($uint:ty) => {
            impl IntoMapKey for $uint {
                fn into_map_key(self) -> String {
                    self.to_string()
                }
            }
        };
    }

    macro_rules! impl_to_map_key_non_zero {
        ($nz:ty) => {
            impl IntoMapKey for $nz {
                fn into_map_key(self) -> String {
                    self.get().to_string()
                }
            }
        };
    }

    impl_to_map_key_uint!(u8);
    impl_to_map_key_uint!(u16);
    impl_to_map_key_uint!(u32);
    impl_to_map_key_uint!(u64);
    impl_to_map_key_uint!(u128);
    impl_to_map_key_uint!(usize);
    impl_to_map_key_non_zero!(std::num::NonZeroU8);
    impl_to_map_key_non_zero!(std::num::NonZeroU16);
    impl_to_map_key_non_zero!(std::num::NonZeroU32);
    impl_to_map_key_non_zero!(std::num::NonZeroU64);
    impl_to_map_key_non_zero!(std::num::NonZeroU128);
    impl_to_map_key_non_zero!(std::num::NonZeroUsize);

    impl IntoMapKey for String {
        fn into_map_key(self) -> String {
            self
        }
    }

    pub trait MapKeyType {
        type MapKeyType;
    }

    #[macro_export]
    macro_rules! MapKeyImpl {
        (
        $(#[$($meta_item:tt)+])*
        $pub:vis struct $Item:ident($Inner:ident);
    ) => {
            impl $crate::map::IntoMapKey for $Item {
                fn into_map_key(self) -> String {
                    self.0.into_map_key()
                }
            }
        };
    }

    pub trait Store: Sized + MapKeyType {
        fn load_at(storage: &dyn ReadonlyStorage, key: Self::MapKeyType) -> Option<Self>;

        fn save_at(&self, storage: &mut dyn MutableStorage, key: Self::MapKeyType);
    }

    pub trait ClearAt: MapKeyType {
        fn clear_at(storage: &mut dyn MutableStorage, key: Self::MapKeyType);
    }

    pub trait LoadAlwaysAt: Sized + MapKeyType {
        fn load_always_at(storage: &dyn ReadonlyStorage, key: Self::MapKeyType) -> Self;
    }

    pub trait ClearAtOrLoadAlwaysAt {}

    #[macro_export]
    macro_rules! store_map_derive_attrs {
        ($Item:ident, custom(map_store(key, $key:ty))) => {
            impl $crate::map::MapKeyType for $Item {
                type MapKeyType = $key;
            }

            impl $crate::map::Store for $Item {
                fn load_at(
                    storage: &dyn $crate::ReadonlyStorage,
                    key: Self::MapKeyType,
                ) -> Option<Self> {
                    storage
                        .get(Self::map_key(key).as_bytes())
                        .map(Self::from_owned_bytes)
                }

                fn save_at(&self, storage: &mut dyn $crate::MutableStorage, key: Self::MapKeyType) {
                    storage.set(
                        Self::map_key(key).as_bytes(),
                        self.to_owned_bytes().as_slice(),
                    );
                }
            }
        };
        ($Item:ident, custom(map_store(always))) => {
            impl $crate::map::ClearAtOrLoadAlwaysAt for $Item {}

            impl $crate::map::LoadAlwaysAt for $Item {
                fn load_always_at(
                    storage: &dyn $crate::ReadonlyStorage,
                    key: Self::MapKeyType,
                ) -> Self {
                    Self::load_at(storage, key).expect("always present in storage")
                }
            }
        };
        ($Item:ident, custom(map_store(clear))) => {
            impl $crate::map::ClearAtOrLoadAlwaysAt for $Item {}

            impl $crate::map::ClearAt for $Item {
                fn clear_at(storage: &mut dyn $crate::MutableStorage, key: Self::MapKeyType) {
                    storage.clear(Self::map_key(key).as_bytes());
                }
            }
        };
        ($_Item:ident, $($_other_meta:tt)+) => {};
    }

    #[macro_export]
    macro_rules! MapStoreImpl {
        (
        $(#[$($meta_item:tt)+])*
        $pub:vis struct $Item:ident($Inner:ident);
    ) => {
            impl $Item {
                $crate::paste! {
                    const KEY_PREFIX: &'static str = concat!(module_path!(), "::", stringify!([< $Item:snake _ $Inner:snake >]));
                }

                fn map_key(key: <Self as $crate::map::MapKeyType>::MapKeyType) -> String {
                    use $crate::map::IntoMapKey;

                    let mut full_key = Self::KEY_PREFIX.to_owned();
                    full_key.push_str("::");
                    full_key.push_str(key.into_map_key().as_str());
                    full_key
                }
            }

            $(
                $crate::store_map_derive_attrs!($Item, $($meta_item)+);
            )*
        };
    }
}

pub mod non_zero {
    pub trait Newtype: Sized {
        type PrimitiveInner;
        type NonZeroInner;

        fn non_zero(self) -> Self::NonZeroInner;

        fn get(self) -> Self::PrimitiveInner;
    }

    pub trait FromNonZero: Sized + Newtype {
        fn from_non_zero<NonZero>(non_zero: NonZero) -> Self
        where
            Self::NonZeroInner: From<NonZero>;
    }

    pub trait CheckedNew: Sized + Newtype {
        fn checked_new<T>(t: T) -> Option<Self>
        where
            Self::PrimitiveInner: From<T>;
    }

    #[macro_export]
    macro_rules! non_zero_newtype_derive_attrs {
        ($Item:ident, custom(non_zero_newtype(from_non_zero))) => {
            impl $crate::non_zero::FromNonZero for $Item {
                fn from_non_zero<NonZero>(non_zero: NonZero) -> Self
                where
                    Self::NonZeroInner: From<NonZero>,
                {
                    Self(Self::NonZeroInner::from(non_zero))
                }
            }
        };
        ($Item:ident, custom(non_zero_newtype(checked_new))) => {
            impl $crate::non_zero::CheckedNew for $Item {
                fn checked_new<T>(t: T) -> Option<Self>
                where
                    Self::PrimitiveInner: From<T>,
                {
                    Self::NonZeroInner::new(Self::PrimitiveInner::from(t)).map(Self)
                }
            }
        };
        ($_Item:ident, $($_other_meta:tt)+) => {};
    }

    #[macro_export]
    macro_rules! NonZeroNewtypeImpl {
        (
        $(#[$($meta_item:tt)+])*
        $pub:vis struct $Newtype:ident($NonZeroInteger:path);
    ) => {
            impl $Newtype {
                fn from_owned_bytes(bytes: Vec<u8>) -> Self {
                    let be_bytes =
                        TryFrom::try_from(bytes).expect("always stored correct amount of bytes");

                    let primative = <Self as $crate::non_zero::Newtype>::PrimitiveInner::from_be_bytes(be_bytes);

                    let non_zero = <Self as $crate::non_zero::Newtype>::NonZeroInner::new(primative).expect("saved primative > 0");

                    Self(non_zero)
                }

                fn to_owned_bytes(&self) -> Vec<u8> {
                    self.0.get().to_be_bytes().to_vec()
                }
            }

            impl $crate::non_zero::Newtype for $Newtype {
                type NonZeroInner = $NonZeroInteger;
                type PrimitiveInner = <Self::NonZeroInner as $crate::Primitive>::Primative;

                fn non_zero(self) -> Self::NonZeroInner {
                    self.0
                }

                fn get(self) -> Self::PrimitiveInner {
                    self.0.get()
                }
            }

            $(
                $crate::non_zero_newtype_derive_attrs!($Newtype, $($meta_item)+);
            )*
        };
    }
}

pub mod uint {
    pub trait Newtype: Sized {
        type PrimitiveInner;
        type NonZeroInner;

        fn get(self) -> Self::PrimitiveInner;

        fn non_zero(self) -> Option<Self::NonZeroInner>;
    }

    pub trait New: Sized + Newtype {
        fn new<T>(t: T) -> Self
        where
            Self::PrimitiveInner: From<T>;
    }

    #[macro_export]
    macro_rules! uint_newtype_derive_attrs {
        ($Item:ident, custom(uint_newtype(new))) => {
            impl $crate::uint::New for $Item {
                fn new<T>(t: T) -> Self
                where
                    Self::PrimitiveInner: From<T>,
                {
                    Self(Self::PrimitiveInner::from(t))
                }
            }
        };
        ($_Item:ident, $($_other_meta:tt)+) => {};
    }

    #[macro_export]
    macro_rules! UintNewtypeImpl {
        (
        $(#[$($meta_item:tt)+])*
        $pub:vis struct $Newtype:ident($Uint:ty);
    ) => {
            impl $Newtype {
                fn from_owned_bytes(bytes: Vec<u8>) -> Self {
                    let be_bytes =
                        TryFrom::try_from(bytes).expect("always stored correct amount of bytes");

                    let primative = <Self as $crate::uint::Newtype>::PrimitiveInner::from_be_bytes(be_bytes);

                    Self(primative)
                }

                fn to_owned_bytes(&self) -> Vec<u8> {
                    self.0.to_be_bytes().to_vec()
                }
            }

            impl $crate::uint::Newtype for $Newtype {
                type PrimitiveInner = $Uint;
                type NonZeroInner = <$Uint as $crate::NonZeroEquivalent>::NonZeroEquivalent;

                fn get(self) -> Self::PrimitiveInner {
                    self.0
                }

                fn non_zero(self) -> Option<Self::NonZeroInner> {
                    Self::NonZeroInner::new(self.0)
                }
            }

            $(
                $crate::uint_newtype_derive_attrs!($Newtype, $($meta_item)+);
            )*
        };
    }
}

pub mod string {
    pub trait Newtype: Sized {
        fn new<S>(s: S) -> Self
        where
            S: Into<String>;

        fn as_str(&self) -> &str;

        fn into_string(self) -> String;
    }

    pub trait New: Sized {
        fn new(s: String) -> Self;
    }

    #[macro_export]
    macro_rules! string_newtype_derive_attrs {
        ($_Item:ident, $($_other_meta:tt)+) => {};
    }

    #[macro_export]
    macro_rules! StringNewtypeImpl {
        (
        $(#[$($meta_item:tt)+])*
        $pub:vis struct $Newtype:ident(String);
    ) => {
            impl $Newtype {
                fn from_owned_bytes(bytes: Vec<u8>) -> Self {
                    String::from_utf8(bytes)
                        .ok()
                        .map(Self)
                        .expect("stored valid utf-8")
                }

                fn to_owned_bytes(&self) -> Vec<u8> {
                    self.0.as_bytes().to_owned()
                }
            }

            impl $crate::string::Newtype for $Newtype {
                fn new<S>(s: S) -> Self
                where
                    S: Into<String> {
                    Self(s.into())
                }

                fn as_str(&self) -> &str {
                    self.0.as_str()
                }

                fn into_string(self) -> String {
                    self.0
                }
            }

            $(
                $crate::string_newtype_derive_attrs!($Newtype, $($meta_item)+);
            )*
        };
    }
}

pub mod prelude {
    pub use crate::item::{Clear, LoadAlways as ItemLoadAlways, Store as ItemStore};
    pub use crate::map::{ClearAt, LoadAlwaysAt, Store as MapStore};
    pub use crate::non_zero::{CheckedNew, FromNonZero, Newtype as NonZeroNewtype};
    pub use crate::string::{New as NewStringNewtype, Newtype as StringNewtype};
    pub use crate::uint::{New as NewUintNewtype, Newtype as UintNewtype};
}
