use std::collections::HashMap;
pub struct DynamicType{
    pointer: *const (),
    size: usize,
    typename: String,
}
impl DynamicType{
    pub fn new<T>(data: &T) -> Self{
        let typename = stringify!(T).to_string();
        let size = std::mem::size_of::<T>();
        let pointer = unsafe {std::mem::transmute(data as *const T)};
        Self {pointer, size, typename}
    }
    pub fn convert<T>(self) -> *mut T
        where T: Sized
    {
        unsafe {std::mem::transmute(self.pointer)}
    }
}
pub trait CollectCallback {
    fn collect(&self) -> HashMap<String, DynamicType>;
}
macro_rules! implcollect {
    ($name:ident, $( $type:ty => $callback:ident ),*) => {
        impl CollectCallback for $name{
            fn collect(&self) -> HashMap<String, DynamicType>{

            }
        }
    }
}