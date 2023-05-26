use bloom::BloomFilter;
use bloomd::bloomd_server::{Bloomd, BloomdServer};
use bloomd::{ContainsRequest, ContainsResponse, InsertRequest, InsertResponse};
use parking_lot::RwLock;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct BloomdService {
    bloom_filter: RwLock<BloomFilter>,
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

        self.bloom_filter.write().insert(&req.get_ref().item);
        Ok(Response::new(bloomd::InsertResponse {}))
    }

    async fn contains(
        &self,
        req: Request<ContainsRequest>,
    ) -> Result<Response<ContainsResponse>, Status> {
        println!("Got a request: {:?}", req);

        Ok(Response::new(bloomd::ContainsResponse {
            contains_item: self.bloom_filter.read().contains(&req.get_ref().item),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allocate Bloom filter
    let bf = BloomFilter::new(100_000, 0.01);
    println!("BloomFilter size={} bytes", bf.size());

    let addr = "[::1]:50051".parse()?;
    Server::builder()
        .add_service(BloomdServer::new(BloomdService {
            bloom_filter: RwLock::new(bf),
        }))
        .serve(addr)
        .await?;

    Ok(())
}
