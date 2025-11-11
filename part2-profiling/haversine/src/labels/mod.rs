use crate::profiling_labels;

profiling_labels! {
    pub enum Labels {
        Args = 1,
        JsonIO,
        JsonParse,
        JsonLookup,
        JsonFree,
        Haversine,
        AfterMath,
        JsonParseUnknown,
    }
}
