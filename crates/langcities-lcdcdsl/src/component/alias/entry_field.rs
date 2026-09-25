use crate::component::{Id, Slug};
use crate::{complex_resource_alias, parse_car};

complex_resource_alias! {
    struct Fields {
        index: Id,
        field: Slug
    }

    id_format:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct IdAliasedEntryField;

    if_display_fmt:
        #[inline]
        iae_displayfmt() {}

    head_format:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct HeadAliasedEntryField;

    hf_display_fmt:
        #[inline]
        hae_display_fmt() {}

    aliased:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum AliasedEntryField;

    ad_display_fmt:
        #[inline]
        ae_display_fmt() {}
    ad_from_str:
        #[inline]
        ae_from_str() {}

    alias:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum EntryFieldAlias;

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
    use crate::component::{EntryFieldAlias, FromFullIdentifier, ToFullIdentifier};

    #[test]
    fn test_valid_entry_field_alias() {
        let cases = [
            "$lang@me.123.ipa",
            "$lang.456.transliteration",
            "$789.oof",
            "$123.456.hello",
        ];

        for case in cases {
            let result = EntryFieldAlias::from_full(case).unwrap();
            assert_eq!(result.to_full(), case);
        }
    }
}
