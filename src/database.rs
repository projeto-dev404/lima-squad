use std::{collections::HashMap, io::SeekFrom, mem::size_of};

use anyhow::Context;
use bytemuck::{from_bytes, try_from_bytes, Pod};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter},
};

pub struct Database {
    data_types: HashMap<&'static str, usize>,
    path_to_data_folder: String,
}

const INDEX_SIZE: usize = size_of::<usize>();

impl Database {
    pub fn new(path_to_data_folder: String) -> Self {
        Self {
            data_types: HashMap::new(),
            path_to_data_folder,
        }
    }

    pub async fn register_type<T: Pod>(&mut self) -> anyhow::Result<File> {
        let tname: &'static str = std::any::type_name::<T>();
        let f = File::create(format!("{}/{tname}", self.path_to_data_folder)).await?;
        self.data_types.insert(tname, size_of::<T>() + INDEX_SIZE);

        Ok(f)
    }

    async fn get_or_register_db<T: Pod>(&mut self) -> anyhow::Result<File> {
        let tname = std::any::type_name::<T>();

        let ret = if self.data_types.contains_key(tname) {
            File::open(format!("{}/{tname}", self.path_to_data_folder)).await?
        } else {
            self.register_type::<T>().await?
        };

        Ok(ret)
    }

    pub async fn save<T: Pod>(&mut self, value: &T) -> anyhow::Result<()> {
        let mut f = self.get_or_register_db::<T>().await?;

        f.seek(SeekFrom::End((INDEX_SIZE + size_of::<T>()) as _))
            .await?;

        let mut buff = [0; INDEX_SIZE];

        let index = if f.read_exact(&mut buff).await.is_ok() {
            usize::from_ne_bytes(buff) + 1
        } else {
            0
        };

        let mut f = BufWriter::new(f);

        f.write(&index.to_ne_bytes()).await?;
        f.write(bytemuck::bytes_of(value)).await?;
        f.flush().await?;

        Ok(())
    }

    async fn prep_before_read<T: Pod>(&self) -> anyhow::Result<(File, usize)> {
        let tname = std::any::type_name::<T>();

        let size_of_type = *self
            .data_types
            .get(tname)
            .context("type has not been registered")?;

        // SAFETY: file existance checked in the code above
        let f = unsafe {
            File::open(format!("{}/{tname}", self.path_to_data_folder))
                .await
                .unwrap_unchecked()
        };

        if f.metadata().await?.len() == 0 {
            anyhow::bail!("table is empty");
        }

        Ok((f, size_of_type))
    }

    pub async fn get<T: Pod>(&self, idx: usize) -> anyhow::Result<T> {
        let (f, size_of_type) = self.prep_before_read::<T>().await?;

        let mut f = BufReader::new(f);
        let mut buffer = Vec::with_capacity(size_of_type);
        f.seek(SeekFrom::Start((size_of_type * (idx - 1)) as _))
            .await?;
        for _ in 0..size_of_type {
            let byte = f.read_u8().await?;
            buffer.push(byte);
        }

        try_from_bytes(&buffer)
            .cloned()
            .map_err(|err| anyhow::anyhow!("failed to convert bytes into data: {err}"))
    }

    pub async fn get_all<T: Pod>(&self) -> anyhow::Result<impl Iterator<Item = T>> {
        let (f, size_of_type) = self.prep_before_read::<T>().await?;

        if f.metadata().await?.len() == 0 {
            anyhow::bail!("table is empty");
        }

        let f = BufReader::new(f);
        let stream = ByteChunkStream::new(f, size_of_type);

        Ok(stream.map(|x| *from_bytes(&x[INDEX_SIZE..])))
    }
}

struct ByteChunkStream<W> {
    buffer: BufReader<W>,
    chunk_size: usize,
    runtime_handle: tokio::runtime::Handle,
}

impl<W> ByteChunkStream<W> {
    fn new(buffer: BufReader<W>, chunk_size: usize) -> Self {
        let runtime_handle = tokio::runtime::Handle::current();

        Self {
            buffer,
            chunk_size,
            runtime_handle,
        }
    }
}

impl<W: AsyncReadExt + Unpin> Iterator for ByteChunkStream<W> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.runtime_handle.block_on(async {
            let mut buffer = Vec::with_capacity(self.chunk_size);

            for _ in 0..self.chunk_size {
                match self.buffer.read_u8().await.ok() {
                    Some(b) => buffer.push(b),
                    None => return None,
                }
            }

            Some(buffer)
        })
    }
}
