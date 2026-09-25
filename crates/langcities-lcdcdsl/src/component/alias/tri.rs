use crate::{
    complex_resource_alias,
    component::{FromFullIdentifier, Id, Slug, ToFullIdentifier},
    parse_car,
};

complex_resource_alias! {
    struct Fields {
        index: Id,
        field: Slug,
        misc: Id,
        other: Slug,
    }

    id_format:
        struct IdAliasedTri;

    if_display_fmt:
        #[inline]
        iat_displayfmt() {}

    head_format:
        struct HeadAliasedTri;

    hf_display_fmt:
        #[inline]
        hat_display_fmt() {}

    aliased:
        enum AliasedTri;

    ad_display_fmt:
        #[inline]
        at_display_fmt() {}
    ad_from_str:
        #[inline]
        at_from_str() {}

    alias:
        enum TriAlias;

    as_display_fmt:
        #[inline]
        ta_display_fmt() {}
    as_from_str:
        #[inline]
        ta_from_str() {}

    impl trait { all }
}

#[test]
fn test_valid_tri_alias() {
    let cases = [
        "$lang@me.123.ipa.123.ipa",
        "$lang.456.transliteration.789.e",
        "$789.oof",
        "$123.456.hello",
    ];

    for case in cases {
        let result = TriAlias::from_full(case).unwrap();
        assert_eq!(result.to_full(), case);
    }
}

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
        iaef_displayfmt() {}

    head_format:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct HeadAliasedEntryField;

    hf_display_fmt:
        #[inline]
        haef_display_fmt() {}

    aliased:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum AliasedEntryField;

    ad_display_fmt:
        #[inline]
        aef_display_fmt() {}
    ad_from_str:
        #[inline]
        aef_from_str() {}

    alias:
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum EntryFieldAlias;

    as_display_fmt:
        #[inline]
        efa_display_fmt() {}
    as_from_str:
        #[inline]
        efa_from_str() {}

    impl trait { all }
}

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
        ea_display_fmt() {}
    as_from_str:
        #[inline]
        ea_from_str() {}
    impl trait { all }
}

#[test]
fn test_valid_entry_alias() {
    let cases = ["$lang@me.123", "$lang.456", "$789", "$123.456"];

    for case in cases {
        let result = EntryAlias::from_full(case).unwrap();
        assert_eq!(result.to_full(), case);
    }
}
