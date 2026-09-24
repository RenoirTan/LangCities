#[macro_export]
macro_rules! impl_ser_display {
    ($typ:ident) => {
        impl serde::ser::Serialize for $typ {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_deser_fromstr {
    ($typ:ident) => {
        impl<'de> serde::de::Deserialize<'de> for $typ {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                String::deserialize(deserializer)
                    .map(|s| {
                        s.parse::<$typ>()
                            .map_err(|e| <D::Error as serde::de::Error>::custom(e))
                    })
                    .flatten()
            }
        }
    };
}

pub use impl_deser_fromstr;
pub use impl_ser_display;

#[macro_export]
macro_rules! impl_ser_tofullid {
    ($typ:ident) => {
        impl serde::ser::Serialize for $typ {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let s =
                    <Self as $crate::component::alias::common::ToFullIdentifier>::to_full(&self);
                serializer.serialize_str(&s)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_deser_fromfullid {
    ($typ:ident) => {
        impl<'de> serde::de::Deserialize<'de> for $typ {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                String::deserialize(deserializer)
                    .map(|s| {
                        <$typ as $crate::component::alias::common::FromFullIdentifier>::from_full(
                            &s,
                        )
                        .map_err(|e| <D::Error as serde::de::Error>::custom(e))
                    })
                    .flatten()
            }
        }
    };
}

pub use impl_deser_fromfullid;
pub use impl_ser_tofullid;
