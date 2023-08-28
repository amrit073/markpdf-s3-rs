use aws_config::load_from_env;
use aws_sdk_s3 as s3;
use aws_sdk_secretsmanager as secretmanager;
use lopdf::{xobject, Document};
use s3::primitives::ByteStream;
use std::error::Error;
use tokio::fs::{self};
use tokio_postgres as pgc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Establish PostgreSQL connection
    let (pgclient, connection) =
        pgc::connect("host=localhost user=postgres dbname=test", pgc::NoTls)
            .await
            .expect("Cannot connect");

    // Spawn the connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Query PostgreSQL and print results
    for row in pgclient.query("SELECT 'amrit';", &[]).await? {
        let name: &str = row.get(0);
        println!("{}", name);
    }
    println!("queried");

    // Load AWS configuration
    let config = load_from_env().await;

    // Download file from S3
    let s3_client = s3::Client::new(&config);
    let get_command = s3_client
        .get_object()
        .set_key(Some("blank.pdf".to_string()))
        .set_bucket(Some("codnivrustbucket".to_string()));

    if let Ok(res) = get_command.send().await {
        let content = res.body.collect().await?.into_bytes();
        fs::write("./s3file.pdf", &content).await?;
        println!("Downloaded file");
    } else {
        println!("Error downloading file");
    }

    // Retrieve secret from AWS Secrets Manager
    let secret_client = secretmanager::Client::new(&config);
    let secret = secret_client
        .get_secret_value()
        .set_secret_id(Some("JWT_SECRET".to_string()))
        .send()
        .await?
        .secret_string
        .unwrap_or_default();
    println!("Retrieved secret: {}", secret);

    // Load and modify PDF
    let mut doc = Document::load("./s3file.pdf")?;
    doc.version = "1.4".to_string();
    let image = xobject::image("./test.png")?;

    // Insert image into PDF pages
    for (_page_number, page_id) in doc.get_pages() {
        if let Err(e) = doc.insert_image(page_id, image.clone(), (10.0, 10.0), (1000.0, 1000.0)) {
            println!("Error inserting image: {}", e);
        } else {
            println!("Image inserted successfully");
        }
    }

    doc.save("modified.pdf")?;
    println!("Modified PDF saved");

    // Upload modified PDF back to S3
    let file = ByteStream::from_path("modified.pdf")
        .await
        .expect("Error reading modified pdf");
    if let Ok(response) = s3_client
        .put_object()
        .set_bucket(Some("codnivrustbucket".to_string()))
        .set_key(Some("modified.pdf".to_string()))
        .set_body(Some(file))
        .send()
        .await
    {
        println!("Uploaded, Version ID: {:?}", response.version_id);
    } else {
        println!("Error uploading");
    }

    Ok(())
}
