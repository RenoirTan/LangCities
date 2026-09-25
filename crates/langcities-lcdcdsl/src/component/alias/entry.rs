use crate::component::Id;
use crate::{complex_resource_alias, parse_car};

complex_resource_alias! {
    struct Fields {
        index: Id
    }

    id_format:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct IdAliasedEntry;

    if_display_fmt:
        #[inline]
        iae_displayfmt() {}

    head_format:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct HeadAliasedEntry;

    hf_display_fmt:
        #[inline]
        hae_display_fmt() {}

    aliased:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum AliasedEntry;

    ad_display_fmt:
        #[inline]
        ae_display_fmt() {}
    ad_from_str:
        #[inline]
        ae_from_str() {}

    alias:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum EntryAlias;

    as_display_fmt:
        #[inline]
        as_display_fmt() {}
    as_from_str:
        #[inline]
        as_from_str() {}
    impl trait { all }
}

#[cfg(test)]
mod test {
    use crate::component::{EntryAlias, FromFullIdentifier, ToFullIdentifier};

    #[test]
    fn test_valid_entry_alias() {
        let cases = ["$lang@me.123", "$lang.456", "$789", "$123.456"];

        for case in cases {
            let result = EntryAlias::from_full(case).unwrap();
            assert_eq!(result.to_full(), case);
        }
    }
}
