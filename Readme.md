# Modify s3's pdf with rust

 Connect to a PostgreSQL database, interact with AWS S3, modify PDF documents, and perform various file operations. 
 Uses the `tokio` runtime for asynchronous programming, the AWS SDK for Rust and the `lopdf` crate for PDF manipulation.

## Workflow

1. Establish a connection to a PostgreSQL database.
2. Download a PDF file from an S3 bucket.
3. Modify the downloaded PDF by inserting an image into its pages.
4. Save the modified PDF locally.
5. Upload the modified PDF back to the S3 bucket.

## Dependencies

- `aws-sdk-s3`: The AWS SDK for Rust, used for interacting with S3.
- `lopdf`: A crate for reading and modifying PDF documents.
- `tokio`: A runtime for writing asynchronous Rust code.

## Memory Usage

![Memory Usage](https://i.imgur.com/KGzSo5c.png)
