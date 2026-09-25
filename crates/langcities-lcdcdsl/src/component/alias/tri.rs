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
        iae_displayfmt() {}

    head_format:
        struct HeadAliasedTri;

    hf_display_fmt:
        #[inline]
        hae_display_fmt() {}

    aliased:
        enum AliasedTri;

    ad_display_fmt:
        #[inline]
        ae_display_fmt() {}
    ad_from_str:
        #[inline]
        ae_from_str() {}

    alias:
        enum TriAlias;

    as_display_fmt:
        #[inline]
        as_display_fmt() {}
    as_from_str:
        #[inline]
        as_from_str() {}

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
