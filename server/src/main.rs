use lib::start;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    start().await.unwrap().await.unwrap();
}
