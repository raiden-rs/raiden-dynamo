use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "Project")]
#[raiden(rename_all = "camelCase")]
pub struct Project {
    #[raiden(partition_key)]
    id: String,
    org_id: String,
    updated_at: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Project::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let cond = Project::key_condition(Project::org_id()).eq("myOrg");
        let res = client
            .query()
            .index("orgIndex")
            .limit(11)
            .key_condition(cond)
            .run()
            .await;
    }
    rt.block_on(example());
}
