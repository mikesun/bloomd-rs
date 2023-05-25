use bloom::BloomFilter;
use bloomd::bloomd_server::{Bloomd, BloomdServer};
use bloomd::{ContainsRequest, ContainsResponse, InsertRequest, InsertResponse};
use std::sync::{Arc, RwLock};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct BloomdService {
    bloom_filter: Arc<RwLock<BloomFilter>>,
}

pub mod bloomd {
    tonic::include_proto!("bloomd");
}

#[tonic::async_trait]
impl Bloomd for BloomdService {
    async fn insert(
        &self,
        req: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        println!("Got a request: {:?}", req);

        match self.bloom_filter.write() {
            Ok(mut bf) => bf.insert(&req.get_ref().item),
            Err(_) => return Err(Status::internal("something bad happened")),
        };
        Ok(Response::new(bloomd::InsertResponse {}))
    }

    async fn contains(
        &self,
        req: Request<ContainsRequest>,
    ) -> Result<Response<ContainsResponse>, Status> {
        println!("Got a request: {:?}", req);

        let res = match self.bloom_filter.read() {
            Ok(bf) => bf.contains(&req.get_ref().item),
            Err(_) => return Err(Status::internal("something bad happened")),
        };

        Ok(Response::new(bloomd::ContainsResponse {
            contains_item: res,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allocate Bloom filter
    let bf = BloomFilter::new(100_000, 0.01);
    println!("BloomFilter size={} bytes", bf.size());

    let addr = "[::1]:50051".parse()?;
    let svc = BloomdService {
        bloom_filter: Arc::new(RwLock::new(bf)),
    };

    Server::builder()
        .add_service(BloomdServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
