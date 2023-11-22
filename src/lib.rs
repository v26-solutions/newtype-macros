#![deny(clippy::all)]
#![warn(clippy::pedantic)]

pub use derive_more;

/// Generates and exports a macro to create newtype wrappers around unsigned integer types.
/// Each generated macro allows for wrapping a specified primitive unsigned integer type,
/// providing additional type safety and domain-specific meaning while retaining the
/// characteristics of the underlying type.
///
/// # Usage
/// - `gen_wrap_uint!(u64);` - Generates a macro for wrapping `u64`.
/// - The generated macro can then be used to define newtypes, e.g., `wrap_u64!(UserId);`.
///
/// The newtype will have traits like `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`,
/// `Ord`, `Display`, `From`, `FromStr`, `Into`, `Add`, `AddAssign`, and `Sum` implemented.
macro_rules! gen_wrap_uint {
    ($int:ty) => {
        paste::paste! {
            #[macro_export]
            macro_rules! [<wrap_ $int>] {
                ($t:ident) => {
                    #[derive(
                        Debug,
                        Default,
                        Clone,
                        Copy,
                        PartialEq,
                        Eq,
                        PartialOrd,
                        Ord,
                        $crate::derive_more::Display,
                        $crate::derive_more::From,
                        $crate::derive_more::FromStr,
                        $crate::derive_more::Into,
                        $crate::derive_more::Add,
                        $crate::derive_more::AddAssign,
                        $crate::derive_more::Sum,
                    )]
                    pub struct $t($int);

                    impl $t {
                        pub const fn zero() -> Self {
                            Self(0)
                        }

                        #[allow(dead_code)]
                        const fn inner(self) -> $int {
                            self.0
                        }

                        pub const fn [< $int >](self) -> $int {
                            self.0
                        }

                        pub const fn is_zero(self) -> bool {
                            self.0 == 0
                        }

                        pub const fn [<into_non_zero_ $int>](self) -> Option<std::num::[< NonZero $int:upper> ]> {
                           std::num::[< NonZero $int:upper> ]::new(self.0)
                        }
                    }

                    impl From<std::num::[< NonZero $int:upper> ]> for $t {
                        fn from(v: std::num::[< NonZero $int:upper> ]) -> $t {
                            $t(v.get())
                        }
                    }
                };
            }
        }
    };
}

/// Creates a newtype wrapper around `String`. The newtype uses `Rc<String>` internally,
/// providing efficient cloning. It's useful for cases where a string type needs to be
/// passed around frequently without the overhead of copying the entire string each time.
///
/// # Usage
/// - `wrap_string!(UserName);` - Creates a `UserName` type wrapping a `String`.
///
/// The newtype will have traits like `Clone`, `Debug`, `PartialEq`, `Display`, `From`,
/// `Into`, and `AsRef` implemented.
#[macro_export]
macro_rules! wrap_string {
    ($t:ident) => {
        #[derive(
            Clone,
            Debug,
            PartialEq,
            $crate::derive_more::Display,
            $crate::derive_more::From,
            $crate::derive_more::Into,
            $crate::derive_more::AsRef,
        )]
        #[as_ref(forward)]
        pub struct $t(::std::rc::Rc<String>);

        impl $t {
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl From<String> for $t {
            fn from(value: String) -> Self {
                Self(::std::rc::Rc::new(value))
            }
        }

        impl From<&str> for $t {
            fn from(value: &str) -> Self {
                Self(::std::rc::Rc::new(value.to_owned()))
            }
        }
    };
}

/// Implements `From` trait for converting one type to another.
/// This macro simplifies the implementation of the `From` trait for conversions between types,
/// especially useful when dealing with multiple newtypes or conversions.
///
/// # Usage
/// - `impl_from!(UserId, from: UserName);` - Implements `From<UserName> for UserId`.
/// - `impl_from!(UserId, from: [UserName, Email]);` - Implements `From` for multiple types.
#[macro_export]
macro_rules! impl_from {
    ($t:ident, from: $from:ident) => {
        impl From<$from> for $t {
            fn from(v: $from) -> $t {
                $t(v.into())
            }
        }
    };

    ($t:ident, from: [$($from:ident),+]) => {
        $(
            impl_from!($t, from: $from);
        )+
    };
}

/// Implements `PartialEq` and `PartialOrd` for comparisons between two different types.
/// This macro is particularly useful when comparing newtypes or domain-specific types
/// that are logically comparable but differ in their actual types.
///
/// # Usage
/// - `impl_ord_eq!(UserId, with: Email);` - Implements comparisons between `UserId` and `Email`.
/// - `impl_ord_eq!(UserId, with: [Email, UserName]);` - Implements comparisons with multiple types.
#[macro_export]
macro_rules! impl_ord_eq {
    ($t:ident, with: $rhs:ident) => {
        impl PartialEq<$rhs> for $t {
            fn eq(&self, other: &$rhs) -> bool {
                self.inner().eq(&other.inner())
            }
        }

        impl PartialEq<$t> for $rhs {
            fn eq(&self, other: &$t) -> bool {
                self.inner().eq(&other.inner())
            }
        }

        impl PartialOrd<$rhs> for $t {
            fn partial_cmp(&self, other: &$rhs) -> Option<std::cmp::Ordering> {
                self.inner().partial_cmp(&other.inner())
            }
        }

        impl PartialOrd<$t> for $rhs {
            fn partial_cmp(&self, other: &$t) -> Option<std::cmp::Ordering> {
                self.inner().partial_cmp(&other.inner())
            }
        }
    };

    ($t:ident, with: [$($rhs:ident),+]) => {
        $(
            impl_ord_eq!($t, with: $rhs);
        )+
    };
}

gen_wrap_uint!(u128);

gen_wrap_uint!(u64);

gen_wrap_uint!(u32);

gen_wrap_uint!(u16);

gen_wrap_uint!(u8);
