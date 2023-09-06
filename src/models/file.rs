use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use rocket::data::Limits;
use rocket::form::{self, DataField, FromFormField};
use rocket::http::ContentType;
use rocket::serde::{Deserialize, Serialize};

use crate::errors::Errors;

#[derive(Debug, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct FileRecord {
    pub hash: String,
    pub name: String,
    pub ext: String,
}

impl FileRecord {
    pub fn from(hex: &[u8], name: String, ext: String) -> Self {
        Self {
            hash: hex::encode(&hex),
            name,
            ext,
        }
    }
}

pub struct HashedFile {
    pub content_type: AllowedContentType,
    pub hash: [u8; 16],
    pub name: String,
}

impl From<HashedFileBuf> for HashedFile {
    fn from(buf: HashedFileBuf) -> Self {
        Self {
            content_type: buf.content_type,
            hash: buf.hash,
            name: buf.name,
        }
    }
}

const DOCX_SUB: &str = "vnd.openxmlformats-officedocument.wordprocessingml.document";

#[derive(Debug)]
pub struct AllowedContentType(ContentType);

impl AllowedContentType {
    pub fn extension(&self) -> Option<String> {
        vec![
            ContentType::PNG,
            ContentType::JPEG,
            ContentType::PDF,
            ContentType::EPUB,
            ContentType::WEBP,
        ]
        .into_iter()
        .find(|c| c == &self.0)
        .map(|c| c.extension().unwrap().to_string())
        .or(match &self.0 {
            c if c == &ContentType::new("application", "x-fictionbook+xml") => {
                Some("fb2".to_owned())
            }
            c if c == &ContentType::new("application", "msword") => Some("doc".to_owned()),
            c if c == &ContentType::new("application", DOCX_SUB) => Some("docx".to_owned()),
            c if c == &ContentType::new("image", "vnd.djvu") => Some("djvu".to_owned()),
            _ => None,
        })
    }

    pub fn extension_err(&self) -> Result<String, Errors> {
        self.extension()
            .ok_or(Errors::Validation("Invalid file type".to_owned()))
    }
}

impl From<ContentType> for AllowedContentType {
    fn from(other: ContentType) -> Self {
        Self(other)
    }
}

/// `FormField` guard. Reads files that are no bigger than 24 mebibytes, otherwise returns a rocket validation error.
/// Only accepts PNG, JPEG, PDF, WEBP, DOC, DOCX, DJVU and FB2 files, and only when their mime type is correctly set. For fb2 it uses `application/x-fictionbook+xml`.
/// Can be saved to a physical location with `persist_to()`.
#[derive(Debug)]
pub struct HashedFileBuf {
    data: Vec<u8>,
    pub content_type: AllowedContentType,
    pub hash: [u8; 16],
    pub name: String,
}

#[rocket::async_trait]
impl<'v> FromFormField<'v> for HashedFileBuf {
    async fn from_data(field: DataField<'v, '_>) -> form::Result<'v, Self> {
        let content_type = AllowedContentType(field.content_type);
        let extension = content_type
            .extension()
            .ok_or(form::Error::validation("Invalid file type"))?;
        warn_!("{}", extension);
        let limits = field.request.limits();
        let limit = limits
            .find(["file", &extension])
            .unwrap_or(limits.find(["file"]).unwrap_or(Limits::FILE));

        let bytes = field.data.open(limit).into_bytes().await?;
        if !bytes.is_complete() {
            Err((None, Some(limit)).into())
        } else if bytes.is_empty() {
            Err(form::Error::validation("Empty files not allowed").into())
        } else {
            let bytes = bytes.into_inner();
            let hash = md5::compute(&bytes).into();
            Ok(Self {
                data: bytes,
                content_type,
                hash,
                name: field.file_name.map_or(hex::encode(hash), |fname| {
                    fname.dangerous_unsafe_unsanitized_raw().to_string()
                }),
            })
        }
    }
}

impl HashedFileBuf {
    /// Persists file to given path and frees it from memory. Returns new path.
    pub fn persist_to<P>(&mut self, path: &P) -> Result<(), io::Error>
    where
        P: AsRef<Path>,
    {
        if self.data.is_empty() {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Can't persist an empty file!",
            ))
        } else {
            let mut file = File::create(path)?;
            file.write_all(&self.data)?;
            self.data = vec![];
            Ok(())
        }
    }
}
