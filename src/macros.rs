macro_rules! dragable! {
    ($name:ident {
        $(
            $field:ident : $type:ty
        ),*
    }) => {
        struct $name {
            $(
                $field: $type
            ),*
        }
        
        
    }
}