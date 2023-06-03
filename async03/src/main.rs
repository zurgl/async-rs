use async03::SlowRead;
use tokio::{fs::File, io::AsyncReadExt, time::Instant};

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let mut buf = vec![0u8; 128 * 1024];
    let f = SlowRead::new(File::open("/dev/urandom").await?);
    tokio::pin!(f);
    let before = Instant::now();
    f.read_exact(&mut buf).await?;
    println!("Read {} bytes in {:?}", buf.len(), before.elapsed());

    let before = Instant::now();
    f.read_exact(&mut buf).await?;
    println!("Read {} bytes in {:?}", buf.len(), before.elapsed());

    Ok(())
}
