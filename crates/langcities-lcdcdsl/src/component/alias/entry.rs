use crate::component::Id;
use crate::error::DslError;
use crate::{complex_resource_alias, parse_car};

complex_resource_alias! {
    id_format:
        struct IdAliasedEntry {
            index: Id
        }

    if_display_fmt:
        iae_displayfmt() {}

    head_format:
        struct HeadAliasedEntry {
            index: Id
        }

    hf_display_fmt: hae_display_fmt() {}

    aliased:
        enum AliasedEntry;

    ad_display_fmt: ae_display_fmt() {}

    alias:
        enum EntryAlias;

    as_display_fmt: as_display_fmt() {}
}

fn linux(s: &str) -> Result<AliasedEntry, DslError> {
    let mut parts = s.rsplit('.').peekable();
    parse_car!(parse_field {
        s: s;
        parts: parts;
        Self: AliasedEntry;
        id_format: IdAliasedEntry;
        index: Id;
    });
    parse_car!(epilogue {
        s: s;
        parts: parts;
        Self: AliasedEntry;
        id_format: IdAliasedEntry;
        head_format: HeadAliasedEntry;
        index: Id
    });
    todo!();
}
