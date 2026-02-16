use crate::profiling_labels;

profiling_labels! {
    pub enum Labels {
        Args = 1,
        PreIO,
        IO,
        JsonParse,
        JsonLookup,
        JsonFree,
        Haversine,
        AfterMath,
        JsonParseUnknown,
    }
}
