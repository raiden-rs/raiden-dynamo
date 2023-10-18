use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "Project")]
#[raiden(rename_all = "camelCase")]
pub struct Project {
    #[raiden(partition_key)]
    pub id: String,
    pub org_id: String,
    pub updated_at: String,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("query_rename=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Project::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let cond = Project::key_condition(Project::org_id()).eq("myOrg");
        let _res = client
            .query()
            .index("orgIndex")
            .limit(11)
            .key_condition(cond)
            .run()
            .await;
    }
    rt.block_on(example());
}
