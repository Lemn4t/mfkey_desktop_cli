use super::session::FlipperSession;
use super::{FlipperError, Result};
use crate::{pb, pb_storage};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
}

impl FlipperSession {
    pub fn storage_list(&mut self, path: &str) -> Result<Vec<DirEntry>> {
        let req = pb_storage::ListRequest {
            path: path.to_string(),
            ..Default::default()
        };
        let parts = self.request_stream(
            pb::main::Content::StorageListRequest(req),
            Duration::from_secs(10),
        )?;

        let mut out = Vec::new();
        for c in parts {
            if let pb::main::Content::StorageListResponse(resp) = c {
                for f in resp.file {
                    out.push(DirEntry {
                        name: f.name,
                        is_dir: f.r#type == 1,
                    });
                }
            }
        }
        Ok(out)
    }

    pub fn storage_read(&mut self, path: &str) -> Result<Vec<u8>> {
        let req = pb_storage::ReadRequest {
            path: path.to_string(),
        };
        let parts = self.request_stream(
            pb::main::Content::StorageReadRequest(req),
            Duration::from_secs(30),
        )?;
        let mut data = Vec::new();
        for c in parts {
            if let pb::main::Content::StorageReadResponse(resp) = c
                && let Some(file) = resp.file
            {
                data.extend_from_slice(&file.data);
            }
        }
        Ok(data)
    }

    pub fn storage_write_with_progress(
        &mut self,
        path: &str,
        data: &[u8],
        mut on_progress: impl FnMut(usize, usize),
    ) -> Result<()> {
        use super::framing::write_message;
        const CHUNK: usize = 512;
        let total = data.len();

        if total <= CHUNK {
            let req = pb_storage::WriteRequest {
                path: path.to_string(),
                file: Some(pb_storage::File {
                    data: data.to_vec(),
                    ..Default::default()
                }),
            };
            self.request(
                pb::main::Content::StorageWriteRequest(req),
                Duration::from_secs(30),
            )?;
            on_progress(total, total);
            return Ok(());
        }

        let id = self.alloc_id_pub();
        let mut offset = 0usize;

        while offset < total {
            let end = (offset + CHUNK).min(total);
            let part = &data[offset..end];
            let has_next = end < total;

            let req = pb_storage::WriteRequest {
                path: path.to_string(),
                file: Some(pb_storage::File {
                    data: part.to_vec(),
                    ..Default::default()
                }),
            };
            let msg = pb::Main {
                command_id: id,
                command_status: 0,
                has_next,
                content: Some(pb::main::Content::StorageWriteRequest(req)),
            };
            write_message(&mut *self.t, &msg)?;
            offset = end;
            on_progress(offset, total);
        }

        let resp = self.recv_for_pub(id, Duration::from_secs(30))?;
        if resp.command_status != 0 {
            return Err(FlipperError::CommandStatus(resp.command_status));
        }
        Ok(())
    }

    pub fn upload_file(
        &mut self,
        path: &str,
        data: &[u8],
        on_progress: impl FnMut(usize, usize),
    ) -> Result<()> {
        let _ = self.storage_delete(path, false);
        self.storage_write_with_progress(path, data, on_progress)
    }

    pub fn storage_delete(&mut self, path: &str, recursive: bool) -> Result<()> {
        let req = pb_storage::DeleteRequest {
            path: path.to_string(),
            recursive,
        };
        self.request(
            pb::main::Content::StorageDeleteRequest(req),
            Duration::from_secs(10),
        )?;
        Ok(())
    }
}
