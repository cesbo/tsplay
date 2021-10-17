use {
    tokio::fs::File,

    super::AsyncStream,
};

impl AsyncStream for File {}
