mod constants;

use aws_config::load_from_env;
use aws_sdk_s3 as s3;
// use aws_sdk_secretsmanager as secretmanager;
use lopdf::{xobject, Document};
use s3::primitives::ByteStream;
use std::error::Error;
use tokio::fs::{self};
use tokio_postgres as pgc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Establish PostgreSQL connection
    let (pgclient, connection) = pgc::connect(constants::PG_CONNECTION_STRING, pgc::NoTls)
        .await
        .expect("Cannot connect");

    // Spawn the connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = pgclient
        .query("SELECT 'blank.pdf';", &[])
        .await
        .expect("cannot read rows");

    let pdf_filename: String = rows[0].get(0);

    // Load AWS configuration
    let config = load_from_env().await;

    // Download file from S3
    let s3_client = s3::Client::new(&config);
    let get_command = s3_client
        .get_object()
        .set_key(Some(pdf_filename))
        .set_bucket(Some(constants::BUCKET_NAME.to_string()));

    if let Ok(res) = get_command.send().await {
        let content = res.body.collect().await?.into_bytes();
        fs::write("/tmp/s3file.pdf", &content).await?;
    } else {
        panic!("Error downloading file");
    }

    // let secret_client = secretmanager::Client::new(&config);
    // let secret = secret_client
    //     .get_secret_value()
    //     .set_secret_id(Some("JWT_SECRET".to_string()))
    //     .send()
    //     .await?
    //     .secret_string
    //     .unwrap_or_default();
    // println!("Retrieved secret: {}", secret);

    // Load and modify PDF
    let mut doc = Document::load("/tmp/s3file.pdf")?;
    doc.version = constants::PDF_VERSION.to_string();
    let image = xobject::image(constants::IMAGE_TO_INSERT).expect("Cannot find image to insert");

    // Insert image into PDF pages
    for (_page_number, page_id) in doc.get_pages() {
        if let Err(e) = doc.insert_image(page_id, image.clone(), (10.0, 10.0), (1000.0, 1000.0)) {
            println!("Error inserting image: {}", e);
        }
    }

    doc.save("/tmp/modified.pdf")?;

    // Upload modified PDF back to S3
    let file = ByteStream::from_path("/tmp/modified.pdf")
        .await
        .expect("Error reading modified pdf");
    if let Ok(_response) = s3_client
        .put_object()
        .set_bucket(Some(constants::BUCKET_NAME.to_string()))
        .set_key(Some("modified.pdf".to_string()))
        .set_body(Some(file))
        .send()
        .await
    {
        println!("Successfully uploaded modified pdf");
    } else {
        println!("Error uploading");
    }

    Ok(())
}
