use jdwp_client_tests::JavaInstance;
use std::io;
use std::time::Duration;
use tracing::info;

#[test_log::test(tokio::test)]
async fn test_connect() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    client.on_event(|e| async move {
        info!("event: {e:?}");
        Ok(())
    }).await;
    tokio::time::sleep(Duration::from_millis(5000)).await;
    Ok(())
}
