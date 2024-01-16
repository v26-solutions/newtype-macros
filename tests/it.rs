use std::num::NonZeroU128;

use expect_test::{expect, Expect};
use macro_rules_attribute::derive;

use newtype_macros::{prelude::*, MapKeyImpl, MapStoreImpl, NonZeroNewtypeImpl, StringNewtypeImpl};
use newtype_macros::{ItemStoreImpl, MutableStorage, ReadonlyStorage, UintNewtypeImpl};

pub fn check(actual: impl std::fmt::Debug, expected: Expect) {
    expected.assert_eq(&format!("{actual:#?}"));
}

#[derive(Default)]
struct SingleCellStore(Option<(Vec<u8>, Vec<u8>)>);

impl SingleCellStore {
    fn key_str(&self) -> Option<&str> {
        self.0
            .as_ref()
            .map(|(k, _)| std::str::from_utf8(k).unwrap())
    }
}

impl ReadonlyStorage for SingleCellStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let (k, v) = self.0.as_ref()?;

        key.eq(k).then_some(v.to_owned())
    }
}

impl MutableStorage for SingleCellStore {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.0 = Some((key.to_owned(), value.to_owned()))
    }

    fn clear(&mut self, key: &[u8]) {
        let Some((k, _)) = self.0.as_ref() else {
            return;
        };

        if key != k {
            return;
        }

        self.0.take();
    }
}

#[derive(Debug, PartialEq, UintNewtypeImpl!, ItemStoreImpl!)]
#[custom(item_store(always))]
#[custom(uint_newtype(new))]
struct FooUint(u64);

#[test]
fn uint_item_storage() {
    let mut storage = SingleCellStore::default();

    let x = FooUint::new(19u8);

    x.save(&mut storage);

    check(
        storage.key_str(),
        expect![[r#"
            Some(
                "it::foo_uint_u64",
            )"#]],
    );

    let x = FooUint::load_always(&storage);

    assert_eq!(x, FooUint(19));
}

#[derive(Debug, PartialEq, NonZeroNewtypeImpl!, ItemStoreImpl!)]
#[custom(item_store(clear))]
#[custom(non_zero_newtype(checked_new))]
#[custom(non_zero_newtype(from_non_zero))]
struct FooNonZero(NonZeroU128);

#[test]
fn non_zero_item_storage() {
    let mut storage = SingleCellStore::default();

    let x = FooNonZero::checked_new(19u8).unwrap();

    x.save(&mut storage);

    check(
        storage.key_str(),
        expect![[r#"
            Some(
                "it::foo_non_zero_non_zero_u128",
            )"#]],
    );

    let x = FooNonZero::load(&storage).unwrap();

    assert_eq!(x, FooNonZero(NonZeroU128::new(19).unwrap()));

    FooNonZero::clear(&mut storage);

    assert!(FooNonZero::load(&storage).is_none());

    assert_eq!(
        FooNonZero::from_non_zero(NonZeroU128::new(19).unwrap()).get(),
        19
    );
}

#[derive(Debug, PartialEq, UintNewtypeImpl!, MapKeyImpl!)]
#[custom(uint_newtype(new))]
struct Baz(u16);

#[derive(Debug, PartialEq, StringNewtypeImpl!, MapStoreImpl!)]
#[custom(map_store(key, (u32, Baz)))]
#[custom(map_store(clear))]
struct BarString(String);

#[derive(Debug, PartialEq, StringNewtypeImpl!, MapStoreImpl!)]
#[custom(map_store(key, String))]
#[custom(map_store(always))]
struct FooString(String);

#[test]
fn string_map_storage() {
    let mut storage = SingleCellStore::default();

    let x = BarString::new("hello");

    x.save_at(&mut storage, (0u32, Baz::new(1u8)));

    check(
        storage.key_str(),
        expect![[r#"
            Some(
                "it::bar_string_string::0:1",
            )"#]],
    );

    let x = BarString::load_at(&storage, (0u32, Baz::new(1u8))).unwrap();

    assert_eq!(x.as_str(), "hello");

    assert!(BarString::load_at(&storage, (1u32, Baz::new(1u8))).is_none());

    BarString::clear_at(&mut storage, (0u32, Baz::new(1u8)));

    assert!(BarString::load_at(&storage, (0u32, Baz::new(1u8))).is_none());

    let x = FooString::new("world");

    x.save_at(&mut storage, "address".to_owned());

    let x = FooString::load_always_at(&storage, "address".to_owned());

    assert_eq!(x.as_str(), "world");
}
