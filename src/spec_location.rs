use crate::error::OpenApiError;
use crate::openapi_spec::OpenApiSpec;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone)]
pub enum OpenApiSpecLocation {
    File(PathBuf),
    Url(Url),
}

impl FromStr for OpenApiSpecLocation {
    type Err = OpenApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            let url =
                Url::parse(s).map_err(|e| OpenApiError::InvalidUrl(format!("Invalid URL: {e}")))?;
            Ok(OpenApiSpecLocation::Url(url))
        } else {
            let path = PathBuf::from(s);
            Ok(OpenApiSpecLocation::File(path))
        }
    }
}

impl OpenApiSpecLocation {
    pub async fn load_spec(&self) -> Result<OpenApiSpec, OpenApiError> {
        match self {
            OpenApiSpecLocation::File(path) => {
                OpenApiSpec::from_file(path.to_str().ok_or_else(|| {
                    OpenApiError::InvalidPath("Invalid file path encoding".to_string())
                })?)
                .await
            }
            OpenApiSpecLocation::Url(url) => OpenApiSpec::from_url(url).await,
        }
    }
}

impl fmt::Display for OpenApiSpecLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenApiSpecLocation::File(path) => write!(f, "{}", path.display()),
            OpenApiSpecLocation::Url(url) => write!(f, "{url}"),
        }
    }
}
