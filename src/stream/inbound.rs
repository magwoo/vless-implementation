pub mod tcp;
pub mod udp;

pub trait InBound {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<Option<usize>>;

    fn write(&mut self, buf: &[u8]) -> anyhow::Result<()>;
}
