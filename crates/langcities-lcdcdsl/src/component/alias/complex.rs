use std::fmt::Display;

use crate::component::{SimpleAliasedResource, Slug};

#[allow(unused)]
fn join_path(path: &[Slug]) -> String {
    path.iter()
        .map(|s| (**s).clone())
        .reduce(|acc, s| acc + "." + &s)
        .unwrap_or_default()
}

/// Represents <slug>[@<user>]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComplexHead {
    Alias(SimpleAliasedResource),
    Slug(Slug),
}

impl Display for ComplexHead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alias(a) => a.fmt(f),
            Self::Slug(a) => a.fmt(f),
        }
    }
}

#[macro_export]
macro_rules! parse_car {
    (parse_field {
        s: $s:ident;
        parts: $parts:ident;
        Self: $self_t:tt;
        id_format: $id_format_t:tt;
        ;
        $($reversed_field:ident: $reversed_field_t:ty),*$(,)?
    }) => {
        $crate::parse_car!(parse_reversed {
            s: $s;
            parts: $parts;
            Self: $self_t;
            id_format: $id_format_t;
            ;
            $($reversed_field: $reversed_field_t),*
        });
    };
    (parse_field {
        s: $s:ident;
        parts: $parts:ident;
        Self: $self_t:tt;
        id_format: $id_format_t:tt;
        $first_field:ident: $first_field_t:ty
        $(, $other_field:ident: $other_field_t:ty)*$(,)?;
        $($reversed_field:ident: $reversed_field_t:ty),*$(,)?
    }) => {
        $crate::parse_car!(parse_field {
            s: $s;
            parts: $parts;
            Self: $self_t;
            id_format: $id_format_t;
            $($other_field: $other_field_t),*;
            $first_field: $first_field_t
            $(, $reversed_field: $reversed_field_t)*
        });
    };
    (parse_reversed {
        s: $s:ident;
        parts: $parts:ident;
        Self: $self_t:tt;
        id_format: $id_format_t:tt;
        $($done_field:ident: $done_field_t:ty),*$(,)?;
        $current_field:ident: $current_field_t:ty$(,)?
    }) => {
        let $current_field: &str = if let Some(next) = $parts.next() {
            if let None = $parts.peek() {
                // next is the last
                if let Ok(id) = next.parse::<$crate::component::id::Id>() {
                    $(let $done_field = $done_field.parse::<$done_field_t>()?;)*
                    return Ok($self_t::IdFormat($id_format_t {
                        id,
                        $current_field: None,
                        $($done_field: Some($done_field)),*
                    }));
                } else {
                    return Err(<$crate::error::DslError as $crate::error::DslErrorTrait>::bad_value_of($s));
                }
            } else {
                next
            }
        } else {
            return Err(<$crate::error::DslError as $crate::error::DslErrorTrait>::bad_value_of($s));
        };
    };
    (parse_reversed {
        s: $s:ident;
        parts: $parts:ident;
        Self: $self_t:tt;
        id_format: $id_format_t:tt;
        $($done_field:ident: $done_field_t:ty),*$(,)?;
        $current_field:ident: $current_field_t:ty,
        $($undone_field:ident: $undone_field_t:ty),*$(,)?
    }) => {
        let $current_field: &str = if let Some(next) = $parts.next() {
            if let None = $parts.peek() {
                // next is the last
                if let Ok(id) = next.parse::<$crate::component::id::Id>() {
                    $(let $done_field = $done_field.parse::<$done_field_t>()?;)*
                    return Ok($self_t::IdFormat($id_format_t {
                        id,
                        $current_field: None,
                        $($undone_field: None,)*
                        $($done_field: Some($done_field),)*
                    }));
                } else {
                    return Err(<$crate::error::DslError as $crate::error::DslErrorTrait>::bad_value_of($s));
                }
            } else {
                next
            }
        } else {
            return Err(<$crate::error::DslError as $crate::error::DslErrorTrait>::bad_value_of($s));
        };
        $crate::parse_car!(parse_reversed {
            s: $s;
            parts: $parts;
            Self: $self_t;
            id_format: $id_format_t;
            $current_field: $current_field_t
            $(, $done_field: $done_field_t)*;
            $($undone_field: $undone_field_t),*
        });
    };
    (epilogue {
        s: $s:ident;
        parts: $parts:ident;
        Self: $self_t:tt;
        id_format: $id_format_t:tt;
        head_format: $head_format_t:tt;
        $($done_field:ident: $done_field_t:ty),*
    }) => {{
        let remainder = $parts.collect::<Vec<&str>>().join(".");
        let premier = remainder.parse::<$crate::component::alias::SimpleResourceAlias>()?;
        $(let $done_field = $done_field.parse::<$done_field_t>()?;)*
        match premier {
            $crate::component::alias::SimpleResourceAlias::Id(id) => {
                let inner = $self_t::IdFormat($id_format_t {
                    id,
                    $($done_field: Some($done_field)),*
                });
                let res: std::result::Result<$self_t, $crate::error::DslError> = Ok(inner);
                res
            },
            $crate::component::alias::SimpleResourceAlias::Alias(alias) => {
                let head = $crate::component::alias::ComplexHead::Alias(alias);
                Ok($self_t::HeadFormat($head_format_t {
                    head,
                    $($done_field),*
                }))
            },
            $crate::component::alias::SimpleResourceAlias::Slug(slug) => {
                let head = $crate::component::alias::ComplexHead::Slug(slug);
                Ok($self_t::HeadFormat($head_format_t {
                    head,
                    $($done_field),*
                }))
            }
        }
    }};
}

#[macro_export]
macro_rules! impl_display_using {
    ($fn:ident for $type:ident) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $fn(self, fmt)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_fromstr_using {
    ($fn:ident for $type:ident) => {
        impl std::str::FromStr for $type {
            type Err = $crate::error::DslError;

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                $fn(s)
            }
        }
    };
}

#[macro_export]
macro_rules! complex_resource_alias {
    {
        struct Fields {
            $(
                $field_name:ident: $field_type:ty
            ),+$(,)?
        }

        id_format:
            $(#[$if_attribute:meta])*
            $if_vis:vis struct $if_name:ident;

        if_display_fmt:
            $(#[$if_display_fmt_attribute:meta])*
            $if_display_fmt_vis:vis $if_display_fmt:ident() {}

        head_format:
            $(#[$hf_attribute:meta])*
            $hf_vis:vis struct $hf_name:ident;

        hf_display_fmt:
            $(#[$hf_display_fmt_attribute:meta])*
            $hf_display_fmt_vis:vis $hf_display_fmt:ident() {}

        aliased:
            $(#[$ad_attribute:meta])*
            $ad_vis:vis enum $ad_name:ident;

        ad_display_fmt:
            $(#[$ad_display_fmt_attribute:meta])*
            $ad_display_fmt_vis:vis $ad_display_fmt:ident() {}

        ad_from_str:
            $(#[$ad_from_str_attribute:meta])*
            $ad_from_str_vis:vis $ad_from_str:ident() {}

        alias:
            $(#[$as_attribute:meta])*
            $as_vis:vis enum $as_name:ident;

        as_display_fmt:
            $(#[$as_display_fmt_attribute:meta])*
            $as_display_fmt_vis:vis $as_display_fmt:ident() {}

        as_from_str:
            $(#[$as_from_str_attribute:meta])*
            $as_from_str_vis:vis $as_from_str:ident() {}

        impl trait { $($t:tt)* }
    } => {
        $(#[$if_attribute])*
        $if_vis struct $if_name {
            $if_vis id: $crate::component::id::Id,
            $(
                $if_vis $field_name: Option<$field_type>
            ),+
        }

        $(#[$if_display_fmt_attribute])*
        $if_display_fmt_vis fn $if_display_fmt(me: &$if_name, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let $if_name {
                id,
                $(
                    $field_name
                ),+
            } = me;

            write!(f, "{}", id)?;
            $(
                if let Some(t) = $field_name {
                    write!(f, ".{}", t)?;
                }
            )*

            Ok(())
        }

        $(#[$hf_attribute])*
        $hf_vis struct $hf_name {
            $hf_vis head: $crate::component::alias::complex::ComplexHead,
            $(
                $hf_vis $field_name: $field_type
            ),+
        }

        $(#[$hf_display_fmt_attribute])*
        $hf_display_fmt_vis fn $hf_display_fmt(me: &$hf_name, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let $hf_name {
                head,
                $(
                    $field_name
                ),+
            } = me;

            write!(f, "{}", head)?;
            $(
                write!(f, ".{}", $field_name)?;
            )*

            Ok(())
        }

        $(#[$ad_attribute])*
        $ad_vis enum $ad_name {
            IdFormat($if_name),
            HeadFormat($hf_name),
        }

        $(#[$ad_display_fmt_attribute])*
        $ad_display_fmt_vis fn $ad_display_fmt(me: &$ad_name, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match me {
                $ad_name::IdFormat(a) => $if_display_fmt(a, f),
                $ad_name::HeadFormat(a) => $hf_display_fmt(a, f),
            }
        }

        $(#[$ad_from_str_attribute])*
        $ad_from_str_vis fn $ad_from_str(s: &str) -> std::result::Result<$ad_name, $crate::error::DslError> {
            let mut parts = s.rsplit('.').peekable();
            parse_car!(parse_field {
                s: s;
                parts: parts;
                Self: $ad_name;
                id_format: $if_name;
                $($field_name: $field_type),*;
            });
            parse_car!(epilogue {
                s: s;
                parts: parts;
                Self: $ad_name;
                id_format: $if_name;
                head_format: $hf_name;
                $($field_name: $field_type),*
            })

        }

        $(#[$as_attribute])*
        $as_vis enum $as_name {
            Id($crate::component::id::Id),
            Alias($ad_name),
        }

        $(#[$as_display_fmt_attribute])*
        $as_display_fmt_vis fn $as_display_fmt(me: &$as_name, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match me {
                $as_name::Id(id) => std::fmt::Display::fmt(id, f),
                $as_name::Alias(a) => $ad_display_fmt(a, f),
            }
        }

        $(#[$as_from_str_attribute])*
        $as_from_str_vis fn $as_from_str(s: &str) -> std::result::Result<$as_name, $crate::error::DslError> {
            s.parse::<$crate::component::id::Id>().map($as_name::Id).or_else(|_| {
                $ad_from_str(s).map($as_name::Alias)
            })
        }

        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
}

#[macro_export]
macro_rules! impl_complex_resource_alias {
    (impl {} where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {};
    (impl {
        all
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_complex_resource_alias!(impl {
            Display for id_format;
            Display for head_format;
            Display for aliased;
            FromStr for aliased;
            Display for alias;
            FromStr for alias;
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
    (impl {
        Display for id_format;
        $($t:tt)*
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_display_using!($if_display_fmt for $if_name);
        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
    (impl {
        Display for head_format;
        $($t:tt)*
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_display_using!($hf_display_fmt for $hf_name);
        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
    (impl {
        Display for aliased;
        $($t:tt)*
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_display_using!($ad_display_fmt for $ad_name);
        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
    (impl {
        Display for alias;
        $($t:tt)*
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_display_using!($as_display_fmt for $as_name);
        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
    (impl {
        FromStr for aliased;
        $($t:tt)*
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_fromstr_using!($ad_from_str for $ad_name);
        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
    (impl {
        FromStr for alias;
        $($t:tt)*
    } where {
        $if_name:ident;
        $if_display_fmt:ident;
        $hf_name:ident;
        $hf_display_fmt:ident;
        $ad_name:ident;
        $ad_display_fmt:ident;
        $ad_from_str:ident;
        $as_name:ident;
        $as_display_fmt:ident;
        $as_from_str:ident;
    }) => {
        $crate::impl_fromstr_using!($as_from_str for $as_name);
        $crate::impl_complex_resource_alias!(impl {
            $($t)*
        } where {
            $if_name;
            $if_display_fmt;
            $hf_name;
            $hf_display_fmt;
            $ad_name;
            $ad_display_fmt;
            $ad_from_str;
            $as_name;
            $as_display_fmt;
            $as_from_str;
        });
    };
}

/*
/// Represents <id>[.<slug>]*
///
/// `N+1` is the maximum length of the complex path.
/// `N` is the maximum number of dots '.'
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComplexIdAliasedResource<const N: usize> {
    pub id: Id,
    pub path: SmallVec<[Slug; N]>,
}

impl<const N: usize> Display for ComplexIdAliasedResource<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.id, join_path(&self.path))
    }
}

/// Represents <slug>[@<user>][.<slug>]*
///
/// `N+1` is the length of the complex path.
/// `N` is the number of dots '.'
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComplexHeadAliasedResource<const N: usize> {
    pub head: ComplexHead,
    pub path: [Slug; N],
}

impl<const N: usize> Display for ComplexHeadAliasedResource<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.head, join_path(&self.path))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComplexAliasedResource<const N: usize> {
    IdFormat(ComplexIdAliasedResource<N>),
    HeadFormat(ComplexHeadAliasedResource<N>),
}

impl<const N: usize> Display for ComplexAliasedResource<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdFormat(a) => a.fmt(f),
            Self::HeadFormat(a) => a.fmt(f),
        }
    }
}

impl<const N: usize> FromStr for ComplexAliasedResource<N> {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts: SmallVec<[&str; N]> = s.rsplitn(N, ".").collect();
        let len = parts.len();
        // extract head from parts
        // leave path inside parts
        // rsplit, so first component is the last element
        let raw_head = if let Some(first) = parts.pop() {
            if len == N {
                let mut splat = first.rsplitn(2, ".");
                let maybe_head = splat.next().unwrap(); // at least one string per slice
                let actual_head = splat.next();
                if let Some(actual_head) = actual_head {
                    parts.push(maybe_head);
                    actual_head
                } else {
                    maybe_head
                }
            } else {
                first
            }
        } else {
            return Err(DslError::bad_value_of(s));
        };
        let sra = SimpleResourceAlias::from_partial(raw_head)?;
        // Complex head must have full length path
        match sra {
            SimpleResourceAlias::Alias(_) | SimpleResourceAlias::Slug(_) if parts.len() < N => {
                return Err(DslError::bad_value_of(s));
            }
            _ => {}
        }
        let mut path = parts
            .into_iter()
            .try_fold(SmallVec::<[Slug; N]>::new(), |mut acc, s| {
                let slug = Slug::from_partial(s)?;
                acc.push(slug);
                Ok(acc)
            })?;
        path.reverse();
        Ok(match sra {
            SimpleResourceAlias::Id(id) => {
                ComplexAliasedResource::IdFormat(ComplexIdAliasedResource { id, path })
            }
            SimpleResourceAlias::Alias(alias) => {
                ComplexAliasedResource::HeadFormat(ComplexHeadAliasedResource {
                    head: ComplexHead::Alias(alias),
                    path: path.into_inner().unwrap(),
                })
            }
            SimpleResourceAlias::Slug(slug) => {
                ComplexAliasedResource::HeadFormat(ComplexHeadAliasedResource {
                    head: ComplexHead::Slug(slug),
                    path: path.into_inner().unwrap(),
                })
            }
        })
    }
}

impl<const N: usize> serde::Serialize for ComplexAliasedResource<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_full())
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for ComplexAliasedResource<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|s| {
                ComplexAliasedResource::from_full(&s)
                    .map_err(|e| <D::Error as serde::de::Error>::custom(e))
            })
            .flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComplexResourceAlias<const N: usize> {
    Id(Id),
    Alias(ComplexAliasedResource<N>),
}

impl<const N: usize> Display for ComplexResourceAlias<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Alias(a) => a.fmt(f),
        }
    }
}

impl<const N: usize> FromStr for ComplexResourceAlias<N> {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>().map(ComplexResourceAlias::Id).or_else(|_| {
            s.parse::<ComplexAliasedResource<N>>()
                .map(ComplexResourceAlias::Alias)
        })
    }
}

impl<const N: usize> serde::Serialize for ComplexResourceAlias<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_full())
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for ComplexResourceAlias<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|s| {
                ComplexResourceAlias::from_full(&s)
                    .map_err(|e| <D::Error as serde::de::Error>::custom(e))
            })
            .flatten()
    }
}

#[cfg(test)]
mod test {
    use crate::component::{
        FromFullIdentifier, SimpleAlias, SimpleResourceAlias, ToFullIdentifier, ToIdentifier,
    };

    #[test]
    fn test_valid_simple_aliases() {
        let cases = ["$me", "$1", "$2489", "$user"];
        for case in cases {
            let alias = SimpleAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
            assert_eq!(alias.to_partial(), &case[1..])
        }
    }

    #[test]
    fn test_invalid_simple_aliases() {
        let cases = ["oof", "123", "$$hi", "$0emfe", "$hello "];
        for case in cases {
            assert!(SimpleAlias::from_full(case).is_err());
        }
    }

    #[test]
    fn test_valid_simple_resource_aliases() {
        let cases = ["$lang@me", "$rust@123", "$slug", "$123"];
        for case in cases {
            let alias = SimpleResourceAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
            assert_eq!(alias.to_partial(), &case[1..])
        }
    }

    #[test]
    fn test_invalid_simple_resource_aliases() {
        let cases = [
            "lang@me", "rust@123", "slug", "123", "$$hi", "$0emfe", "$hello ", "$1@23",
        ];
        for case in cases {
            assert!(SimpleResourceAlias::from_full(case).is_err());
        }
    }

    /*
    #[test]
    fn test_valid_complex_resource_aliases() {
        fn test<const N: usize>(case: &str) {
            let alias = ComplexResourceAlias::<N>::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
            assert_eq!(alias.to_partial(), &case[1..])
        }
        test::<1>("$lang@me.hi");
    }
    */
}
*/
