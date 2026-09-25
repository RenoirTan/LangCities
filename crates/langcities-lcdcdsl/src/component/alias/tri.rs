use crate::{
    complex_resource_alias,
    component::{Id, Slug},
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
        pub struct IdAliasedTri;

    if_display_fmt:
        #[inline]
        iae_displayfmt() {}

    head_format:
        pub struct HeadAliasedTri;

    hf_display_fmt:
        #[inline]
        hae_display_fmt() {}

    aliased:
        pub enum AliasedTri;

    ad_display_fmt:
        #[inline]
        ae_display_fmt() {}
    ad_from_str:
        #[inline]
        ae_from_str() {}

    alias:
        pub enum TriAlias;

    as_display_fmt:
        #[inline]
        as_display_fmt() {}
    as_from_str:
        #[inline]
        as_from_str() {}

    impl trait { all }
}
