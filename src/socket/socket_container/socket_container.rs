pub trait ISocketContainer {
    fn intialize(&self);
    fn connect(&self);
    fn disconnect(&self);
    fn add_symbol(&self);
    fn remove_symbol(&self);
}
