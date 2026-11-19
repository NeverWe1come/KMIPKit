pub trait TransportFactory {
    fn create<T: kmipkit_transport::Transport>(transport: T);
}
