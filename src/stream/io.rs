// use std::pin::Pin;
// use std::sync::Arc;
// use std::task::{Context, Poll};
// use tokio::io::{AsyncRead, AsyncWrite};
// use tokio::sync::Mutex;

// pub trait IOBound: Sized {
//     async fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<usize>;

//     async fn write(&mut self, buf: &[u8]) -> anyhow::Result<usize>;

//     async fn split(self) -> (HalfReader<Self>, HalfWriter<Self>) {
//         let inner = Arc::new(Mutex::new(self));

//         (HalfReader(Arc::clone(&inner)), HalfWriter(inner))
//     }
// }

// pub struct HalfReader<T>(Arc<Mutex<T>>);

// pub struct HalfWriter<T>(Arc<Mutex<T>>);

// impl<T: IOBound> HalfReader<T> {
//     pub async fn read(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
//         let mut inner = self.0.lock().await;

//         inner.read(buf).await
//     }
// }

// impl<T: IOBound> AsyncRead for IOBoundWrapper<T> {
//     fn poll_read(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> Poll<std::io::Result<()>> {
//         match self.iobounded.poll_read_ready(cx) {
//             Poll::Ready(Ok(())) => {}
//             Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
//             Poll::Pending => return Poll::Pending,
//         }

//         let readed = match self.iobounded.try_read(buf.initialize_unfilled()) {
//             Ok(Some(readed)) => readed,
//             Ok(None) => return Poll::Pending,
//             Err(err) => return Poll::Ready(Err(std::io::Error::other(err))),
//         };

//         buf.advance(readed);

//         Poll::Ready(Ok(()))
//     }
// }

// impl<T: IOBound> AsyncWrite for IOBoundWrapper<T> {
//     fn poll_write(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &[u8],
//     ) -> Poll<Result<usize, std::io::Error>> {
//         match self.iobounded.poll_write_ready(cx) {
//             Poll::Ready(Ok(())) => {}
//             Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
//             Poll::Pending => return Poll::Pending,
//         };

//         let writed = match self.iobounded.try_write(buf) {
//             Ok(Some(readed)) => readed,
//             Ok(None) => return Poll::Pending,
//             Err(err) => return Poll::Ready(Err(std::io::Error::other(err))),
//         };

//         Poll::Ready(Ok(writed))
//     }

//     fn poll_flush(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> Poll<Result<(), std::io::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn poll_shutdown(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> Poll<Result<(), std::io::Error>> {
//         Box::pin(self.iobounded.shutdown())
//             .as_mut()
//             .poll(cx)
//             .map_err(|err| std::io::Error::other(err))
//     }
// }
