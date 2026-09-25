use crate::component::{Id, Slug};
use crate::error::DslError;
use crate::{complex_resource_alias, parse_car};

complex_resource_alias! {
    id_format:
        struct IdAliasedTri {
            index: Id,
            field: Slug,
            misc: Id,
            other: Slug
        }

    if_display_fmt:
        iae_displayfmt() {}

    head_format:
        struct HeadAliasedTri {
            index: Id,
            field: Slug,
            misc: Id,
            other: Slug
        }

    hf_display_fmt: hae_display_fmt() {}

    aliased:
        enum AliasedTri;

    ad_display_fmt: ae_display_fmt() {}

    alias:
        enum TriAlias;

    as_display_fmt: as_display_fmt() {}
}

fn parse_tri(s: &str) -> Result<AliasedTri, DslError> {
    let mut parts = s.rsplit('.').peekable();
    parse_car!(parse_field {
        s: s;
        parts: parts;
        Self: AliasedTri;
        id_format: IdAliasedTri;
        index: Id,
        field: Slug,
        misc: Id,
        other: Slug;
    });
    parse_car!(epilogue {
        s: s;
        parts: parts;
        Self: AliasedTri;
        id_format: IdAliasedTri;
        head_format: HeadAliasedTri;
        index: Id,
        field: Slug,
        misc: Id,
        other: Slug
    })
}
