// Copyright 2021 The Engula Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::PathBuf;

use futures::TryStreamExt;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

use super::{codec, segment_stream::SegmentStream};
use crate::{Error, Event, Result, ResultStream, Timestamp};

pub struct Segment {
    path: PathBuf,
    file: File,
    offset: usize,
    last_timestamp: Option<Timestamp>,
}

impl Segment {
    pub async fn open(
        path: impl Into<PathBuf>,
        mut last_timestamp: Option<Timestamp>,
    ) -> Result<Self> {
        let path = path.into();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        let offset = file.metadata().await?.len() as usize;

        // Recovers the last timestamp.
        let mut stream = SegmentStream::open(&path, offset, None).await?;
        while let Some(events) = stream.try_next().await? {
            for event in events {
                last_timestamp = Some(event.ts);
            }
        }

        Ok(Self {
            path,
            file,
            offset,
            last_timestamp,
        })
    }

    pub async fn seal(mut self) -> Result<Timestamp> {
        let ts = self.last_timestamp.ok_or_else(|| {
            Error::Unknown("should not seal a segment with no timestamp".to_owned())
        })?;
        // Records the last timestamp at the file footer.
        codec::write_footer(&mut self.file, ts).await?;
        self.file.sync_data().await?;
        Ok(ts)
    }

    pub async fn read_events(&self, ts: Timestamp) -> Result<ResultStream<Vec<Event>>> {
        SegmentStream::open(&self.path, self.offset, Some(ts)).await
    }

    pub async fn append_event(&mut self, event: Event) -> Result<usize> {
        if let Some(last_ts) = self.last_timestamp {
            if event.ts <= last_ts {
                return Err(Error::InvalidArgument(format!(
                    "event timestamp {:?} <= last event timestamp {:?}",
                    event.ts, last_ts,
                )));
            }
        }
        let size = codec::write_event(&mut self.file, &event).await?;
        self.file.flush().await?;
        self.offset += size;
        self.last_timestamp = Some(event.ts);
        Ok(self.offset)
    }
}
